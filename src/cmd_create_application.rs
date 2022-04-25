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
use std::path::{Path, PathBuf};

use git2::{Error, Repository};

use crate::api::{ApplicationCreateResponse, CapsuleApi};
use crate::CliError;

impl From<git2::Error> for CliError {
    fn from(e: Error) -> Self {
        CliError { message: e.to_string() }
    }
}

pub fn handle<P, A>(application_directory: P, application_name: Option<String>, api: &A) -> Result<ApplicationCreateResponse, CliError>
    where P: AsRef<Path>, A: CapsuleApi {
    let create_response = api.create_application(application_name)?;

    if is_git_repository(&application_directory) {
        let repo = Repository::open(&application_directory)?;

        let remotes = repo.remotes()?;
        let capsule_remote = remotes.iter().flatten().find(|it| { *it == "capsule" });

        if capsule_remote.is_none() {
            repo.remote("capsule", &create_response.git_repo)?;
        }
    }

    Ok(create_response)
}

fn is_git_repository<P: AsRef<Path>>(application_directory: P) -> bool {
    PathBuf::new().join(application_directory).join(".git").exists()
}

#[cfg(test)]
mod tests {
    use git2::Repository as Git;
    use mockall::{predicate::*};
    use tempdir::TempDir;

    use crate::api::MockCapsuleApi;
    use crate::cmd_create_application::{handle, is_git_repository};

    use super::*;

    #[test]
    fn should_create_application_if_application_directory_is_not_a_git_repository() {
        let application_directory = TempDir::new(".").unwrap();

        let mock_api = mock_create_application_api();

        let result = handle(application_directory.path(), None, &mock_api);

        assert_eq!(result.is_ok(), true);
        let response = result.ok().unwrap();
        assert_eq!(response.name, "first_capsule_application");
        assert_eq!(response.url, "https://first-capsule-application.capsuleapp.cyou");
        assert_eq!(response.git_repo, "https://git.capsuleapp.cyou/first_capsule_user/first-capsule-application.git");
        assert_eq!(is_git_repository(application_directory), false);
    }

    #[test]
    fn should_create_application_if_application_directory_is_a_git_repository() {
        let application_directory = TempDir::new(".").unwrap();

        let _ = Git::init(&application_directory);

        let mock_api = mock_create_application_api();

        let result = handle(application_directory.path(), None, &mock_api);
        assert_eq!(result.is_ok(), true);

        let response = result.ok().unwrap();
        assert_eq!(response.name, "first_capsule_application");
        assert_eq!(response.name, "first_capsule_application");
        assert_eq!(response.url, "https://first-capsule-application.capsuleapp.cyou");
        assert_eq!(response.git_repo, "https://git.capsuleapp.cyou/first_capsule_user/first-capsule-application.git");
    }

    #[test]
    fn should_add_remote_git_repository_if_application_directory_is_a_git_repository() {
        let application_directory = TempDir::new(".").unwrap();

        let git_repo = Git::init(&application_directory).unwrap();

        let mock_api = mock_create_application_api();

        let _ = handle(application_directory.path(), None, &mock_api);

        let capsule_remote = git_repo.find_remote("capsule").unwrap();
        assert_eq!(capsule_remote.name().unwrap(), "capsule");
        assert_eq!(capsule_remote.url().unwrap(), "https://git.capsuleapp.cyou/first_capsule_user/first-capsule-application.git");
    }

    #[test]
    fn should_create_application_if_git_remote_repository_was_present() {
        let application_directory = TempDir::new(".").unwrap();

        let git_repo = Git::init(&application_directory).unwrap();
        git_repo.remote("capsule", "https://git.capsuleapp.cyou/first_capsule_user/first_capsule_application")
            .expect("could not add git remote");

        let mock_api = mock_create_application_api();

        let result = handle(application_directory.path(), None, &mock_api);

        assert_eq!(result.is_ok(), true);
    }

    fn mock_create_application_api() -> MockCapsuleApi {
        let mut mock_api = MockCapsuleApi::new();
        mock_api
            .expect_create_application()
            .with(eq(None))
            .times(1)
            .returning(|_| Ok(ApplicationCreateResponse {
                name: "first_capsule_application".to_string(),
                url: "https://first-capsule-application.capsuleapp.cyou".to_string(),
                git_repo: "https://git.capsuleapp.cyou/first_capsule_user/first-capsule-application.git".to_string(),
            }));
        mock_api
    }
}