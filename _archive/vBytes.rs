#[derive(Debug, Clone, strum::Display)]
pub enum VersionedBytesError {
	InvalidBase32,
	InvalidVersion,
}
