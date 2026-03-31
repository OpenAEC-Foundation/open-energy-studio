import { useState, useRef, useEffect } from 'react';
import { DialogShell } from '../DialogShell';
import { useI18n } from '../../../i18n/i18n';
import { version } from '../../../../package.json';
import './FeedbackDialog.css';

const API_URL = 'https://open-feedback-studio.pages.dev/api/feedback';
const APP_ID = 'open-energy-studio';
const MAX_IMAGES = 3;
const MAX_TOTAL_SIZE = 1024 * 1024; // 1MB
const MAX_MESSAGE = 5000;
const MIN_MESSAGE = 10;

const SENTIMENT_LABELS: Record<number, string> = { 1: 'Frustrated', 2: 'Neutral', 3: 'Happy' };

interface FeedbackDialogProps {
  onClose: () => void;
}

export function FeedbackDialog({ onClose }: FeedbackDialogProps) {
  const { t } = useI18n();

  const [email, setEmail] = useState(() => localStorage.getItem('feedback-email') || '');
  const [fullName, setFullName] = useState(() => localStorage.getItem('feedback-name') || '');
  const [category, setCategory] = useState('general');
  const [message, setMessage] = useState('');
  const [images, setImages] = useState<{ file: File; url: string }[]>([]);
  const [sentiment, setSentiment] = useState<number | null>(null);
  const [status, setStatus] = useState<'idle' | 'submitting' | 'success' | 'error'>('idle');
  const [errorMsg, setErrorMsg] = useState('');

  const fileInputRef = useRef<HTMLInputElement>(null);
  const imagesRef = useRef(images);
  imagesRef.current = images;

  useEffect(() => {
    return () => {
      imagesRef.current.forEach(img => URL.revokeObjectURL(img.url));
    };
  }, []);

  const isValidEmail = () => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.trim());

  const canSubmit = () =>
    isValidEmail() &&
    message.length >= MIN_MESSAGE &&
    message.length <= MAX_MESSAGE &&
    status !== 'submitting';

  const charCountWarning = message.length >= 4500;

  function handleAttach() {
    if (images.length >= MAX_IMAGES) return;
    fileInputRef.current?.click();
  }

  function handleFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const files = Array.from(e.target.files || []);
    if (!files.length) return;

    const currentSize = images.reduce((sum, img) => sum + img.file.size, 0);
    const remaining = MAX_IMAGES - images.length;
    const toAdd = files.slice(0, remaining);

    const newImages: { file: File; url: string }[] = [];
    let newSize = currentSize;

    for (const file of toAdd) {
      if (!file.type.startsWith('image/')) continue;
      if (newSize + file.size > MAX_TOTAL_SIZE) break;
      newSize += file.size;
      newImages.push({ file, url: URL.createObjectURL(file) });
    }

    setImages(prev => [...prev, ...newImages]);
    e.target.value = '';
  }

  function removeImage(index: number) {
    setImages(prev => {
      const updated = [...prev];
      URL.revokeObjectURL(updated[index].url);
      updated.splice(index, 1);
      return updated;
    });
  }

  function resetForm() {
    setCategory('general');
    setMessage('');
    images.forEach(img => URL.revokeObjectURL(img.url));
    setImages([]);
    setSentiment(null);
    setStatus('idle');
    setErrorMsg('');
  }

  async function handleSubmit() {
    if (!canSubmit()) return;

    setStatus('submitting');
    setErrorMsg('');

    try {
      const sentimentValue = sentiment;
      const sentimentLabel = sentimentValue ? SENTIMENT_LABELS[sentimentValue] : undefined;

      const emailVal = email.trim();
      const nameVal = fullName.trim() || undefined;

      let response: Response;

      if (images.length > 0) {
        const formData = new FormData();
        formData.append('app', APP_ID);
        formData.append('email', emailVal);
        if (nameVal) formData.append('fullname', nameVal);
        formData.append('category', category);
        formData.append('message', message.trim());
        if (sentimentLabel) formData.append('sentiment', sentimentLabel);
        formData.append('appVersion', version);
        images.forEach(img => {
          formData.append('images', img.file);
        });

        response = await fetch(API_URL, {
          method: 'POST',
          body: formData,
        });
      } else {
        response = await fetch(API_URL, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            app: APP_ID,
            email: emailVal,
            fullname: nameVal,
            category,
            message: message.trim(),
            sentiment: sentimentLabel,
            appVersion: version,
          }),
        });
      }

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }

      // Remember email and name for next time
      localStorage.setItem('feedback-email', email.trim());
      localStorage.setItem('feedback-name', fullName.trim());

      setStatus('success');
    } catch (e) {
      console.error('Feedback submission failed:', e);
      setStatus('error');
      setErrorMsg(t('feedback.errorGeneric'));
    }
  }

  const categories = [
    { key: 'general', label: t('feedback.categoryGeneral') },
    { key: 'bug', label: t('feedback.categoryBug') },
    { key: 'feature', label: t('feedback.categoryFeature') },
  ];

  const sentiments = [
    { value: 1, emoji: '\u{1F61E}', label: t('feedback.sentimentFrustrated') },
    { value: 2, emoji: '\u{1F610}', label: t('feedback.sentimentNeutral') },
    { value: 3, emoji: '\u{1F60A}', label: t('feedback.sentimentHappy') },
  ];

  return (
    <DialogShell
      title={t('feedback.title')}
      onClose={onClose}
      className="feedback-dialog"
      footer={null}
    >
      {status === 'success' ? (
        <div className="feedback-success">
          <h3>{t('feedback.successTitle')}</h3>
          <p>{t('feedback.successMessage')}</p>
          <button className="btn btn-primary feedback-submit-btn" onClick={resetForm}>
            {t('feedback.sendAnother')}
          </button>
        </div>
      ) : (
        <div className="feedback-form">
          {/* Email & Name */}
          <div className="feedback-section">
            <div className="dialog-field">
              <label>{t('feedback.email')} <span className="feedback-required">*</span></label>
              <input
                type="email"
                placeholder={t('feedback.emailPlaceholder')}
                value={email}
                onChange={e => setEmail(e.target.value)}
              />
            </div>
            <div className="dialog-field">
              <label>{t('feedback.fullName')}</label>
              <input
                type="text"
                placeholder={t('feedback.fullNamePlaceholder')}
                value={fullName}
                onChange={e => setFullName(e.target.value)}
              />
            </div>
          </div>

          {/* Category */}
          <div className="feedback-section">
            <div className="feedback-categories">
              {categories.map(cat => (
                <button
                  key={cat.key}
                  className={`feedback-category-btn${category === cat.key ? ' active' : ''}`}
                  onClick={() => setCategory(cat.key)}
                >
                  {cat.label}
                </button>
              ))}
            </div>
          </div>

          {/* Message */}
          <div className="feedback-section">
            <div className="dialog-field">
              <textarea
                className="feedback-message"
                placeholder={t('feedback.messagePlaceholder')}
                maxLength={MAX_MESSAGE}
                value={message}
                onChange={e => setMessage(e.target.value)}
              />
              <div className={`feedback-char-count${charCountWarning ? ' warning' : ''}`}>
                {message.length} / {MAX_MESSAGE}
              </div>
            </div>
          </div>

          {/* Image Attachments */}
          <div className="feedback-section">
            <div className="feedback-images">
              {images.map((img, i) => (
                <div key={img.url} className="feedback-image-thumb">
                  <img src={img.url} alt="" />
                  <button className="feedback-image-remove" onClick={() => removeImage(i)}>
                    &times;
                  </button>
                </div>
              ))}
            </div>
            {images.length < MAX_IMAGES && (
              <button className="feedback-attach-btn" onClick={handleAttach}>
                {t('feedback.attachImages')}
              </button>
            )}
            <input
              ref={fileInputRef}
              type="file"
              accept="image/*"
              multiple
              style={{ display: 'none' }}
              onChange={handleFileChange}
            />
            <div className="feedback-hint">{t('feedback.imageLimit')}</div>
          </div>

          {/* Sentiment */}
          <div className="feedback-section">
            <div className="feedback-hint">{t('feedback.sentiment')}</div>
            <div className="feedback-sentiment">
              {sentiments.map(s => (
                <button
                  key={s.value}
                  className={`feedback-sentiment-btn${sentiment === s.value ? ' active' : ''}`}
                  onClick={() => setSentiment(sentiment === s.value ? null : s.value)}
                  title={s.label}
                >
                  {s.emoji}
                </button>
              ))}
            </div>
          </div>

          {/* Submit */}
          <button
            className="btn btn-primary feedback-submit-btn"
            disabled={!canSubmit()}
            onClick={handleSubmit}
          >
            {status === 'submitting' ? t('feedback.submitting') : t('feedback.submit')}
          </button>

          {status === 'error' && (
            <div className="feedback-error">{errorMsg}</div>
          )}
        </div>
      )}
    </DialogShell>
  );
}
