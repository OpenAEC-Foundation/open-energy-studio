import { Component, type ErrorInfo, type ReactNode } from 'react';
import { useI18n } from '../../i18n/i18n';

function ErrorFallback({ message, onRetry }: { message: string; onRetry: () => void }) {
  const { t } = useI18n();
  return (
    <div className="error-boundary" role="alert" style={{ padding: 16, fontSize: 13 }}>
      <strong>{t('errorBoundary.title')}</strong>
      <p style={{ margin: '6px 0' }}>{t('errorBoundary.body')}</p>
      <code style={{ display: 'block', fontSize: 11, opacity: 0.8, wordBreak: 'break-word' }}>{message}</code>
      <button type="button" className="btn btn-sm" style={{ marginTop: 8 }} onClick={onRetry}>
        {t('errorBoundary.retry')}
      </button>
    </div>
  );
}

interface Props {
  children: ReactNode;
  /** A change of this value (for example the project) clears the error. */
  resetKey?: unknown;
}

interface State {
  error: Error | null;
  key: unknown;
}

/**
 * Keeps one failing view from blanking the whole application: the error is
 * shown in place of that view, with a retry, and the rest stays usable.
 */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null, key: this.props.resetKey };

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { error };
  }

  static getDerivedStateFromProps(props: Props, state: State): Partial<State> | null {
    return props.resetKey !== state.key ? { error: null, key: props.resetKey } : null;
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('View error', error, info.componentStack);
  }

  render() {
    if (this.state.error) {
      return <ErrorFallback message={this.state.error.message} onRetry={() => this.setState({ error: null })} />;
    }
    return this.props.children;
  }
}
