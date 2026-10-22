use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use theseus::error_code;

#[macro_use]
pub mod astralrinth;
pub mod auth;
pub mod import;
pub mod install;
pub mod instance;
pub mod jre;
pub mod logs;
pub mod metadata;
pub mod minecraft_skins;
pub mod mr_auth;
pub mod onboarding_checklist;
pub mod process;
pub mod reports;
pub mod settings;
pub mod shortcuts;
pub mod tags;
mod thumbnails;
pub mod users;
pub mod utils;

pub mod cache;
pub mod files;
pub mod friends;
pub mod worlds;

mod oauth_utils;

pub type Result<T> = std::result::Result<T, CommandError>;

#[derive(Debug)]
pub struct CommandError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for CommandError {
	fn from(error: E) -> Self {
		Self(error.into())
	}
}

impl Serialize for CommandError {
	fn serialize<S>(
		&self,
		serializer: S,
	) -> std::result::Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		tracing::warn!("{:?}", self.0);

		let mut state = serializer.serialize_struct("CommandError", 2)?;
		state.serialize_field("message", &format!("{:#}", self.0))?;
		state.serialize_field("code", error_code(&self.0))?;
		state.end()
	}
}

#[cfg(test)]
mod tests {
	use super::CommandError;

	#[test]
	fn astralrinth_external_library_error_retains_its_command_code() {
		let error = anyhow::Error::from(
			theseus::ErrorKind::ExternalAuthLibraryNotInstalled {
				provider_name: "Ely.by".to_string(),
			},
		)
		.context("preparing external Minecraft account");
		let value = serde_json::to_value(CommandError::from(error)).unwrap();

		assert_eq!(value["code"], "external_auth_library_not_installed");
		assert!(value["message"].as_str().unwrap().contains("Ely.by"));
	}
}
