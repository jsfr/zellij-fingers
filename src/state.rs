#[derive(Debug, Clone, PartialEq)]
pub enum PluginPhase {
    WaitingForPermissions,
    Capturing,
    Hinting,
    Done,
}
