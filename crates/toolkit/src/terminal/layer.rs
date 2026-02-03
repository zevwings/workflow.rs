//! Spinner 感知的 tracing Layer
//!
//! 在输出日志时自动暂停和恢复 spinner，避免输出冲突。

use std::io::{self, Write};

use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

use super::coordinator::{resume_spinner, suspend_spinner};

/// Spinner 感知的 Writer
///
/// 在写入前暂停 spinner，写入后恢复 spinner。
pub struct SpinnerAwareWriter<W> {
    inner: W,
}

impl<W: Write> Write for SpinnerAwareWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        suspend_spinner();
        let result = self.inner.write(buf);
        // 注意：不在这里恢复，因为可能有多次 write 调用
        result
    }

    fn flush(&mut self) -> io::Result<()> {
        let result = self.inner.flush();
        resume_spinner();
        result
    }
}

/// Spinner 感知的 MakeWriter
pub struct SpinnerAwareMakeWriter;

impl<'a> MakeWriter<'a> for SpinnerAwareMakeWriter {
    type Writer = SpinnerAwareWriter<io::Stderr>;

    fn make_writer(&'a self) -> Self::Writer {
        SpinnerAwareWriter {
            inner: io::stderr(),
        }
    }
}

/// Spinner 感知的 Layer
///
/// 包装底层的 fmt Layer，在日志事件输出时协调 spinner。
pub struct SpinnerAwareLayer<S, L> {
    inner: L,
    _marker: std::marker::PhantomData<S>,
}

impl<S, L> SpinnerAwareLayer<S, L> {
    /// 创建新的 SpinnerAwareLayer
    pub fn new(inner: L) -> Self {
        Self {
            inner,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S, L> Layer<S> for SpinnerAwareLayer<S, L>
where
    S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
    L: Layer<S>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        suspend_spinner();
        self.inner.on_event(event, ctx);
        resume_spinner();
    }

    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        self.inner.on_new_span(attrs, id, ctx);
    }

    fn on_record(
        &self,
        span: &tracing::span::Id,
        values: &tracing::span::Record<'_>,
        ctx: Context<'_, S>,
    ) {
        self.inner.on_record(span, values, ctx);
    }

    fn on_follows_from(
        &self,
        span: &tracing::span::Id,
        follows: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        self.inner.on_follows_from(span, follows, ctx);
    }

    fn on_enter(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
        self.inner.on_enter(id, ctx);
    }

    fn on_exit(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
        self.inner.on_exit(id, ctx);
    }

    fn on_close(&self, id: tracing::span::Id, ctx: Context<'_, S>) {
        self.inner.on_close(id, ctx);
    }
}
