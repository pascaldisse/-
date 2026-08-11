use clap::ValueEnum;

/// 音高の環量子化律。
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum 律 {
    八家,
    十二平均律,
}
