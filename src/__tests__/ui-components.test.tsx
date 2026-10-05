import { describe, expect, it, vi } from 'vitest';
import { useState } from 'react';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '../i18n/I18nProvider';
import i18next from '../i18n/i18n';
import { en } from '../i18n/en';
import { nl } from '../i18n/nl';
import {
  Banner, Button, ConfirmProvider, DataTable, EmptyState, ErrorState, Field, FileButton, IconButton, IssueList,
  NumberInput, Segmented, StatusPill, Stepper, Tabs, ToastProvider, TriState, useConfirm, useToast,
} from '../components/ui';

function wrap(ui: React.ReactElement) {
  return render(<I18nProvider><ToastProvider><ConfirmProvider>{ui}</ConfirmProvider></ToastProvider></I18nProvider>);
}

describe('NumberInput', () => {
  function Harness({ initial = null as number | null | undefined, optional = false, onValue = (_: unknown) => {} }) {
    const [value, setValue] = useState<number | null | undefined>(initial);
    return (
      <Field label="Rc">
        <NumberInput value={value} optional={optional} unit="m²K/W" step={0.1}
          onChange={(next) => { setValue(next); onValue(next); }} />
      </Field>
    );
  }

  it('accepts a comma and a point and reports a number', async () => {
    const user = userEvent.setup();
    const onValue = vi.fn();
    wrap(<Harness onValue={onValue} />);
    const input = screen.getByLabelText('Rc');
    await user.type(input, '0,25');
    expect(onValue).toHaveBeenLastCalledWith(0.25);
    await user.clear(input);
    await user.type(input, '4.7');
    expect(onValue).toHaveBeenLastCalledWith(4.7);
    expect(input).toHaveAttribute('inputmode', 'decimal');
  });

  it('reports null when cleared, or undefined when optional', async () => {
    const user = userEvent.setup();
    const required = vi.fn();
    const { unmount } = wrap(<Harness initial={3} onValue={required} />);
    await user.clear(screen.getByLabelText('Rc'));
    expect(required).toHaveBeenLastCalledWith(null);
    unmount();
    const optional = vi.fn();
    wrap(<Harness initial={3} optional onValue={optional} />);
    await user.clear(screen.getByLabelText('Rc'));
    expect(optional).toHaveBeenLastCalledWith(undefined);
  });

  it('marks text that is not a number as invalid without reporting it', async () => {
    const user = userEvent.setup();
    const onValue = vi.fn();
    wrap(<Harness initial={1} onValue={onValue} />);
    const input = screen.getByLabelText('Rc');
    await user.clear(input);
    onValue.mockClear();
    await user.type(input, 'x');
    expect(onValue).not.toHaveBeenCalled();
    expect(input).toHaveAttribute('aria-invalid', 'true');
  });

  it('steps with the arrow keys; Shift multiplies by ten', async () => {
    const user = userEvent.setup();
    const onValue = vi.fn();
    wrap(<Harness initial={1} onValue={onValue} />);
    const input = screen.getByLabelText('Rc');
    input.focus();
    await user.keyboard('{ArrowUp}');
    expect(onValue).toHaveBeenLastCalledWith(1.1);
    await user.keyboard('{Shift>}{ArrowDown}{/Shift}');
    expect(onValue).toHaveBeenLastCalledWith(0.1);
  });

  it('shows the value with a decimal comma in Dutch and the unit inside the field', async () => {
    await act(async () => { await i18next.changeLanguage('nl'); });
    try {
      wrap(<Harness initial={4.7} />);
      expect(screen.getByLabelText('Rc')).toHaveValue('4,7');
      expect(screen.getByText('m²K/W')).toBeInTheDocument();
    } finally {
      await act(async () => { await i18next.changeLanguage('en'); });
    }
  });
});

describe('Field', () => {
  it('links the error with aria-describedby and keeps role="alert"', () => {
    wrap(<Field label="U-waarde" reference="8.2" error="Te hoog"><input /></Field>);
    const input = screen.getByLabelText(/U-waarde/);
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('Te hoog');
    expect(input).toHaveAttribute('aria-describedby', alert.id);
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByText(/8\.2/)).toBeInTheDocument();
  });
});

describe('Segmented and TriState', () => {
  it('is a radio group that moves with the arrow keys', async () => {
    const user = userEvent.setup();
    function Harness() {
      const [value, setValue] = useState<'wall' | 'roof' | 'floor'>('wall');
      return (
        <Segmented aria-label="Soort" value={value} onChange={setValue}
          options={[{ value: 'wall', label: 'Wand' }, { value: 'roof', label: 'Dak' }, { value: 'floor', label: 'Vloer' }]} />
      );
    }
    wrap(<Harness />);
    const group = screen.getByRole('radiogroup', { name: 'Soort' });
    expect(within(group).getByRole('radio', { name: 'Wand' })).toHaveAttribute('aria-checked', 'true');
    within(group).getByRole('radio', { name: 'Wand' }).focus();
    await user.keyboard('{ArrowRight}');
    expect(within(group).getByRole('radio', { name: 'Dak' })).toHaveAttribute('aria-checked', 'true');
    expect(within(group).getByRole('radio', { name: 'Dak' })).toHaveFocus();
  });

  it('maps Yes / No / Unknown to true / false / null', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    wrap(<TriState aria-label="Bypass" value={null} onChange={onChange} />);
    expect(screen.getByRole('radio', { name: 'Unknown' })).toHaveAttribute('aria-checked', 'true');
    await user.click(screen.getByRole('radio', { name: 'Yes' }));
    expect(onChange).toHaveBeenLastCalledWith(true);
    await user.click(screen.getByRole('radio', { name: 'No' }));
    expect(onChange).toHaveBeenLastCalledWith(false);
  });
});

describe('Buttons, pills and banners', () => {
  it('keeps the width stable while loading and disables the button', () => {
    wrap(<Button loading icon={<span data-testid="icon" />}>Herberekenen</Button>);
    const button = screen.getByRole('button', { name: /Herberekenen/ });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('aria-busy', 'true');
    expect(screen.queryByTestId('icon')).toBeNull();
  });

  it('requires a label on icon buttons and uses it as tooltip', () => {
    wrap(<IconButton aria-label="Sluiten" icon={<span />} />);
    expect(screen.getByRole('button', { name: 'Sluiten' })).toHaveAttribute('title', 'Sluiten');
  });

  it('shows status with icon and text', () => {
    const { container } = wrap(<StatusPill tone="err">Voldoet niet</StatusPill>);
    expect(screen.getByText('Voldoet niet')).toBeInTheDocument();
    expect(container.querySelector('.ui-pill--err svg')).not.toBeNull();
  });

  it('gives error banners and error states role="alert" and info banners role="status"', () => {
    wrap(
      <>
        <Banner tone="err">Kern weigert</Banner>
        <Banner tone="unv">Onverifieerd</Banner>
        <ErrorState title="Mislukt" />
        <EmptyState title="Nog geen zones" />
      </>,
    );
    expect(screen.getAllByRole('alert').map((element) => element.textContent)).toEqual(['Kern weigert', 'Mislukt']);
    expect(screen.getAllByRole('status').map((element) => element.textContent)).toEqual(['Onverifieerd', 'Nog geen zones']);
  });
});

describe('Toasts', () => {
  function Trigger({ tone }: { tone: 'success' | 'error' }) {
    const toast = useToast();
    return <button type="button" onClick={() => toast.show({ tone, title: `Bericht ${tone}`, message: 'details' })}>show</button>;
  }

  it('closes success toasts automatically and keeps errors until closed', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    try {
      wrap(<><Trigger tone="success" /><Trigger tone="error" /></>);
      const [success, error] = screen.getAllByRole('button', { name: 'show' });
      fireEvent.click(success);
      fireEvent.click(error);
      expect(screen.getByText('Bericht success')).toBeInTheDocument();
      expect(screen.getByRole('alert')).toHaveTextContent('Bericht error');
      act(() => { vi.advanceTimersByTime(7000); });
      expect(screen.queryByText('Bericht success')).toBeNull();
      expect(screen.getByText('Bericht error')).toBeInTheDocument();
      fireEvent.click(within(screen.getByRole('alert')).getByRole('button', { name: 'Close' }));
      expect(screen.queryByText('Bericht error')).toBeNull();
    } finally {
      vi.useRealTimers();
    }
  });

  it('shows at most three toasts', () => {
    function Many() {
      const toast = useToast();
      return <button type="button" onClick={() => { for (let i = 1; i <= 5; i += 1) toast.show({ tone: 'error', title: `T${i}` }); }}>many</button>;
    }
    wrap(<Many />);
    fireEvent.click(screen.getByRole('button', { name: 'many' }));
    expect(screen.getAllByRole('alert').map((element) => element.querySelector('strong')?.textContent)).toEqual(['T3', 'T4', 'T5']);
  });
});

describe('ConfirmDialog', () => {
  function Closer({ onChoice }: { onChoice: (choice: string) => void }) {
    const confirm = useConfirm();
    return (
      <button type="button" onClick={async () => onChoice(await confirm({
        title: 'Niet-opgeslagen wijzigingen', message: 'Opslaan?', confirmLabel: 'Opslaan', denyLabel: 'Niet opslaan',
      }))}>sluiten</button>
    );
  }

  it('offers Save, Don\'t save and Cancel and resolves with the choice', async () => {
    const user = userEvent.setup();
    const onChoice = vi.fn();
    wrap(<Closer onChoice={onChoice} />);
    await user.click(screen.getByRole('button', { name: 'sluiten' }));
    const dialog = screen.getByRole('dialog', { name: 'Niet-opgeslagen wijzigingen' });
    expect(within(dialog).getByRole('button', { name: 'Opslaan' })).toBeInTheDocument();
    expect(within(dialog).getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
    await user.click(within(dialog).getByRole('button', { name: 'Niet opslaan' }));
    await waitFor(() => expect(onChoice).toHaveBeenCalledWith('deny'));
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('treats Escape as cancel', async () => {
    const user = userEvent.setup();
    const onChoice = vi.fn();
    wrap(<Closer onChoice={onChoice} />);
    await user.click(screen.getByRole('button', { name: 'sluiten' }));
    await user.keyboard('{Escape}');
    await waitFor(() => expect(onChoice).toHaveBeenCalledWith('cancel'));
  });
});

describe('Tabs, Stepper, DataTable, IssueList', () => {
  it('implements the tabs pattern with arrow keys', async () => {
    const user = userEvent.setup();
    function Harness() {
      const [active, setActive] = useState<'a' | 'b'>('a');
      return <Tabs aria-label="Weergave" active={active} onChange={setActive} tabs={[{ id: 'a', label: 'Overzicht' }, { id: 'b', label: 'Per dienst' }]} />;
    }
    wrap(<Harness />);
    screen.getByRole('tab', { name: 'Overzicht' }).focus();
    await user.keyboard('{ArrowRight}');
    expect(screen.getByRole('tab', { name: 'Per dienst' })).toHaveAttribute('aria-selected', 'true');
  });

  it('marks the current step', () => {
    wrap(<Stepper aria-label="Verwarming" steps={[
      { id: 'gen', label: 'Opwekking', status: 'done' },
      { id: 'dis', label: 'Distributie', status: 'current' },
    ]} />);
    expect(screen.getByText('Distributie').closest('[aria-current]')).toHaveAttribute('aria-current', 'step');
  });

  it('puts the unit in the header and activates rows with Enter', async () => {
    const user = userEvent.setup();
    const onActivate = vi.fn();
    wrap(<DataTable caption="Vlakken" rowKey={(row) => row.id} onActivate={onActivate}
      rows={[{ id: 's1', name: 'Gevel N', area: 30 }, { id: 's2', name: 'Dak', area: 52 }]}
      columns={[
        { key: 'name', header: 'Naam', render: (row) => row.name },
        { key: 'area', header: 'Oppervlak', unit: 'm²', numeric: true, render: (row) => row.area },
      ]} />);
    expect(screen.getByRole('columnheader', { name: /Oppervlak \(m²\)/ })).toBeInTheDocument();
    const rows = screen.getAllByRole('row').slice(1);
    rows[0].focus();
    await user.keyboard('{ArrowDown}{Enter}');
    expect(onActivate).toHaveBeenCalledWith({ id: 's2', name: 'Dak', area: 52 });
  });

  it('lists issues with translated title, location, code and Go to', async () => {
    const user = userEvent.setup();
    const onGoTo = vi.fn();
    wrap(<IssueList onGoTo={onGoTo} issues={[{ code: 'nta_calculation_block_missing', severity: 'error', path: 'ntaCalculation' }]} />);
    expect(screen.getByText('ntaCalculation')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: /Go to/ }));
    expect(onGoTo).toHaveBeenCalled();
  });
});

describe('FileButton', () => {
  it('replaces the native Browse text with a translated button and keeps the input labelled', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    wrap(<><label htmlFor="f">Bestand</label><FileButton id="f" onChange={onChange} /></>);
    expect(screen.getByRole('button', { name: 'Choose file…' })).toBeInTheDocument();
    expect(screen.getByText('No file chosen')).toBeInTheDocument();
    await user.upload(screen.getByLabelText('Bestand'), new File(['{}'], 'project.json', { type: 'application/json' }));
    expect(onChange).toHaveBeenCalled();
    expect(screen.getByText('project.json')).toBeInTheDocument();
  });
});

describe('i18n keys of the redesign', () => {
  const keys = [
    'ui.yes', 'ui.no', 'ui.unknown', 'ui.confirm', 'ui.chooseFile', 'ui.noFileChosen', 'ui.goTo',
    'ui.severity.error', 'ui.severity.warning', 'ui.severity.info', 'ui.stale.title', 'ui.stale.text',
    'app.toast.importFailed', 'app.toast.openFailed', 'app.toast.kernelChanged', 'app.toast.inputChanged',
    'app.toast.relabelMigrated', 'app.unsaved.title', 'app.unsaved.message', 'app.unsaved.save', 'app.unsaved.discard',
    'app.untitled', 'model3d.noData', 'model3d.resetView', 'model3d.exportIFC', 'model3d.dragToRotate',
  ];

  it('exist in Dutch and English', () => {
    const dutch = nl as Record<string, string>;
    const english = en as Record<string, string>;
    for (const key of keys) {
      expect(dutch[key], key).toBeTruthy();
      expect(english[key], key).toBeTruthy();
    }
  });

  it('has no English feedback strings left in the Dutch UI', () => {
    const dutch = nl as Record<string, string>;
    expect(dutch['feedback.sendFeedback']).toBe('Feedback sturen');
    expect(dutch['feedback.submit']).not.toMatch(/Feedback$/);
  });

  it('every model3d key used in the 3D view is translated', async () => {
    const { readFileSync } = await import('fs');
    const { resolve } = await import('path');
    const source = readFileSync(resolve(__dirname, '../components/Building3DView/Building3DView.tsx'), 'utf-8');
    const used = Array.from(source.matchAll(/t\('(model3d\.[A-Za-z0-9_.]+)'/g)).map((match) => match[1]);
    expect(used.length).toBeGreaterThan(0);
    for (const key of used) {
      expect((nl as Record<string, string>)[key], key).toBeTruthy();
      expect((en as Record<string, string>)[key], key).toBeTruthy();
    }
  });
});
