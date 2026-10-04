import { useI18n } from '../../i18n/i18n';
import { kernelCodeLabel } from '../../i18n/format';
import { kernelDetailText } from '../../i18n/kernelDetail';
import './KernelCode.css';

interface KernelCodeProps {
  code: string;
  /** i18n prefixes tried in order; the default covers gaps, warnings and kernel issues. */
  prefixes?: string[];
}

/** A translated kernel code with the raw code as a small reference; the bare code when untranslated. */
export function KernelCode({ code, prefixes }: KernelCodeProps) {
  const { t } = useI18n();
  const label = kernelCodeLabel(t, code, prefixes);
  if (!label.known) return <strong className="kernel-code"><code>{code}</code></strong>;
  return <strong className="kernel-code"><span>{label.text}</span> <code className="kernel-code-ref">{code}</code></strong>;
}

/** The kernel's technical detail: a translated sentence when known, else the raw text marked as technical. */
export function KernelDetail({ detail }: { detail?: string | null }) {
  const { t, locale } = useI18n();
  if (!detail) return null;
  const { text, translated } = kernelDetailText(detail, t, locale);
  if (translated) return <small className="kernel-code-detail">{text}</small>;
  return <small className="kernel-code-detail" lang="en">
    <span className="kernel-code-detail-label">{t('kernel.detail.technical')}:</span> {text}
  </small>;
}
