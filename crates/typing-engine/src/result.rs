#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineInputResult {
    /// 入力が受け入れられた。ただしまだ完了していない。
    Accepted,

    /// 入力が受け入れられ、タイピングが完了した。
    Completed,

    /// 入力が拒否された。
    Rejected,

    /// 既に完了しているため入力を受け付けられない。
    AlreadyCompleted,
}
