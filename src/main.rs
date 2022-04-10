// Copyright 2022 the original author or authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use std::io::Write;
use std::time::Duration;

use clap::{Parser, Subcommand};

use capsule::{CliError, cmd_create_application};
use capsule::api::CapsuleApi;
use capsule::api::http::HttpCapsuleApi;

#[derive(Parser)]
#[clap(name = "capsule")]
#[clap(about = "CLI to interact with Capsule", long_about = None)]
#[clap(version = "1.0")]
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// create application
    Create {
        /// application name
        name: Option<String>
    }
}

fn execute_command(args: &Cli, api: &impl CapsuleApi, writer: &mut impl Write) {
    match &args.command {
        Commands::Create { name } => {
            write!(writer, "Creating application... ").expect("could not print");
            let result = cmd_create_application::handle(".", name.clone(), api);

            match result {
                Err(CliError { message }) => writeln!(writer, "{}", message).expect("could not print"),
                Ok(response) => {
                    writeln!(writer, "done, {}", response.name).expect("could not print");
                    writeln!(writer, "url: {}", response.url).expect("could not print");
                    writeln!(writer, "git: {}", response.git_repo).expect("could not print")
                }
            }
        }
    };
}

fn main() {
    let args: Cli = Cli::parse();

    let api = HttpCapsuleApi {
        uri: "http://api.capsuleapp.cyou".to_string(),
        timeout: Duration::from_secs(5),
    };

    execute_command(&args, &api, &mut std::io::stdout());
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};

    use capsule::api::ApplicationCreateResponse;
    use capsule::api::http::HttpCapsuleApi;

    use crate::{Cli, execute_command};

    use super::*;

    #[async_std::test]
    async fn should_print_url_and_git_repo_if_create_application_successfully() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/applications"))
            .respond_with(ResponseTemplate::new(201)
                .set_body_json(ApplicationCreateResponse {
                    name: "first_capsule_application".to_string(),
                    url: "https://first-capsule-application.capsuleapp.cyou".to_string(),
                    git_repo: "https://git.capsuleapp.cyou/first-capsule-application.git".to_string(),
                }))
            .mount(&mock_server)
            .await;
        let api = HttpCapsuleApi { uri: mock_server.uri(), timeout: Duration::from_secs(5) };

        let args = Cli {
            command: Commands::Create { name: None }
        };

        let mut output: Vec<u8> = vec![];

        execute_command(&args, &api, &mut output);

        let output_strings = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_strings.split("\n").collect();

        assert_eq!(lines[0], "Creating application... done, first_capsule_application");
        assert_eq!(lines[1], "url: https://first-capsule-application.capsuleapp.cyou");
        assert_eq!(lines[2], "git: https://git.capsuleapp.cyou/first-capsule-application.git");
    }
}
