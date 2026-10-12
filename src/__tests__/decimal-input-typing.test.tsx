import { afterEach, describe, expect, it, vi } from 'vitest';
import { useState } from 'react';
import { act, fireEvent, screen } from '@testing-library/react';
import i18next from 'i18next';
import { DecimalInput, NumberInput } from '../components/ui';
import { parseDecimal } from '../i18n/format';
import { UValueCalculator } from '../components/UValueCalculator/UValueCalculator';
import { renderWithProviders, userEvent } from './test-utils';

// Twelfth review of the DecimalInput rollout: typing must never store a number the
// user did not type, the arrow keys keep to min/max, and every locale's display can
// be edited again.

function Parent({ initial, handler, min, max, step }: {
  initial: number | string;
  handler: (value: string, previous: number | string) => number | string;
  min?: number; max?: number; step?: number;
}) {
  const [value, setValue] = useState<number | string>(initial);
  return <>
    <label>Amount<DecimalInput min={min} max={max} step={step} value={value}
      onChange={(event) => setValue(handler(event.target.value, value))} /></label>
    <output data-testid="stored">{String(value)}</output>
  </>;
}

const stored = () => screen.getByTestId('stored').textContent;

describe('typing into a field whose parent rejects a prefix', () => {
  it('keeps the typed text while focused and shows the parent value again on blur', () => {
    renderWithProviders(<Parent initial={5} handler={(value, previous) => (Number(value) >= 0 ? Number(value) : previous)} />);
    const input = screen.getByRole('spinbutton', { name: 'Amount' });
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '-3' } });
    expect(input).toHaveValue('-3');
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(stored()).toBe('5');
    fireEvent.blur(input);
    expect(input).toHaveValue('5');
    expect(input).not.toHaveAttribute('aria-invalid');
  });

  it('stores 0,04 for λ in the U-value calculator, not 8.04', async () => {
    const user = userEvent.setup();
    renderWithProviders(<UValueCalculator />);
    const input = screen.getAllByLabelText(/^λ \(W\/mK\) 1/)[0] as HTMLInputElement;
    await user.tripleClick(input);
    await user.keyboard('0,04');
    expect(input.value).toBe('0,04');
    expect(input).not.toHaveAttribute('aria-invalid');
    await user.tab();
    expect(input.value).toBe('0.04');
  });

  it('lets 0,5 be typed where the parent falls back to 0.8 for a zero (ThermalBridgeCalculator foundation depth)', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Parent initial={0.8} handler={(value) => parseFloat(value) || 0.8} />);
    const input = screen.getByRole('spinbutton') as HTMLInputElement;
    await user.tripleClick(input);
    await user.keyboard('0,5');
    expect(input.value).toBe('0,5');
    expect(stored()).toBe('0.5');
    await user.tab();
    expect(input.value).toBe('0.5');
  });

  it('types 0,35 character by character with a parseFloat || 0 parent', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Parent initial={5} handler={(value) => parseFloat(value) || 0} />);
    const input = screen.getByRole('spinbutton');
    await user.clear(input);
    await user.type(input, '0,35');
    expect(input).toHaveValue('0,35');
    expect(stored()).toBe('0.35');
  });
});

describe('separators that do not form a number', () => {
  it('rejects mixed separators that do not group thousands', () => {
    expect(parseDecimal('0.8,04')).toBeNaN();
    expect(parseDecimal('1,23.4')).toBeNaN();
    expect(parseDecimal('12.34,5')).toBeNaN();
    expect(parseDecimal('1.234,5')).toBe(1234.5);
    expect(parseDecimal('1.234.567,25')).toBe(1234567.25);
    expect(parseDecimal('1,234,567.25')).toBe(1234567.25);
  });

  it('treats an overflowing value as no number and passes nothing on', () => {
    expect(parseDecimal('1e999')).toBeNaN();
    const spy = vi.fn((value: string) => value);
    renderWithProviders(<Parent initial="" handler={spy} />);
    const input = screen.getByRole('spinbutton');
    fireEvent.change(input, { target: { value: '1e999' } });
    expect(spy).not.toHaveBeenCalled();
    expect(input).toHaveAttribute('aria-invalid', 'true');
  });
});

describe('arrow keys', () => {
  it('stay at min when stepping down', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Parent initial={0} min={0} max={90} step={1} handler={(value) => parseFloat(value) || 0} />);
    const input = screen.getByRole('spinbutton');
    await user.click(input);
    await user.keyboard('{ArrowDown}');
    expect(stored()).toBe('0');
    expect(input).toHaveValue('0');
  });

  it('stay at max when stepping up, with the field in step with the parent', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Parent initial={100} max={100} step={1}
      handler={(value, previous) => (Number(value) <= 100 ? Number(value) : previous)} />);
    const input = screen.getByRole('spinbutton');
    await user.click(input);
    await user.keyboard('{ArrowUp}');
    expect(stored()).toBe('100');
    expect(input).toHaveValue('100');
  });

  it('keep NumberInput within min and max too', async () => {
    const user = userEvent.setup();
    function Number01() {
      const [value, setValue] = useState<number | null | undefined>(1);
      return <label>Share<NumberInput min={0} max={1} step={0.1} value={value} onChange={setValue} /></label>;
    }
    renderWithProviders(<Number01 />);
    const input = screen.getByLabelText('Share');
    await user.click(input);
    await user.keyboard('{ArrowUp}');
    expect(input).toHaveValue('1');
  });
});

describe('spinbutton values for assistive technology', () => {
  it('exposes the current value and the bounds', () => {
    renderWithProviders(<Parent initial={12.5} min={0} max={90} handler={(value) => Number(value)} />);
    const input = screen.getByRole('spinbutton');
    expect(input).toHaveAttribute('aria-valuenow', '12.5');
    expect(input).toHaveAttribute('aria-valuemin', '0');
    expect(input).toHaveAttribute('aria-valuemax', '90');
  });
});

describe('numerals from other scripts', () => {
  afterEach(async () => {
    await act(async () => { await i18next.changeLanguage('en'); });
  });

  it('reads Persian, Arabic-Indic and full-width digits and separators', () => {
    expect(parseDecimal('۱۲۳۴٫۵')).toBe(1234.5);
    expect(parseDecimal('١٢٣٤٫٥')).toBe(1234.5);
    expect(parseDecimal('١٬٢٣٤٫٥')).toBe(1234.5);
    expect(parseDecimal('‎−۳')).toBe(-3);
    expect(parseDecimal('‎-3')).toBe(-3);
    expect(parseDecimal('０，３５')).toBe(0.35);
    expect(parseDecimal('１２．５')).toBe(12.5);
    expect(parseDecimal('－２')).toBe(-2);
  });

  for (const locale of ['fa', 'ar']) {
    it(`shows an editable value in ${locale}`, async () => {
      await act(async () => { await i18next.changeLanguage(locale); });
      renderWithProviders(<Parent initial={-1234.5} handler={(value) => Number(value)} />);
      const input = screen.getByRole('spinbutton') as HTMLInputElement;
      expect(input.value).not.toMatch(/[٠-٩۰-۹]/);
      expect(parseDecimal(input.value)).toBe(-1234.5);
    });
  }
});
