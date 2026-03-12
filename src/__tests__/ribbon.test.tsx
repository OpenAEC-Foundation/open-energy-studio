/**
 * Ribbon — functional tests
 *
 * Tests tab switching, button clicks, dialog opening, and calculate action.
 */
import { describe, it, expect, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { Ribbon } from '../components/Ribbon/Ribbon';

function makeProps(overrides: Partial<Parameters<typeof Ribbon>[0]> = {}) {
  return {
    onOpenDialog: vi.fn(),
    onCalculate: vi.fn(),
    onNewProject: vi.fn(),
    onSaveProject: vi.fn(),
    onOpenProject: vi.fn(),
    onExportReport: vi.fn(),
    onExportIFC: vi.fn(),
    onExportModelIFC: vi.fn(),
    onPrintReport: vi.fn(),
    onTogglePreview: vi.fn(),
    onExportUNIEC3: vi.fn(),
    onImportUNIEC3: vi.fn(),
    onExportVABI: vi.fn(),
    onImportVABI: vi.fn(),
    onOpenAppMenu: vi.fn(),
    ...overrides,
  };
}

describe('Ribbon', () => {
  it('renders the File tab', () => {
    renderWithProviders(<Ribbon {...makeProps()} />);
    expect(screen.getByText(/File/i, { selector: '.ribbon-tab.file-tab' })).toBeInTheDocument();
  });

  it('renders all main tabs', () => {
    renderWithProviders(<Ribbon {...makeProps()} />);
    expect(screen.getByText(/Home/i, { selector: '.ribbon-tab' })).toBeInTheDocument();
    expect(screen.getByText(/Building Envelope/i, { selector: '.ribbon-tab' })).toBeInTheDocument();
    expect(screen.getByText(/Installations/i, { selector: '.ribbon-tab' })).toBeInTheDocument();
    expect(screen.getByText(/Renewables/i, { selector: '.ribbon-tab' })).toBeInTheDocument();
    expect(screen.getByText(/Results/i, { selector: '.ribbon-tab' })).toBeInTheDocument();
    expect(screen.getByText(/Report/i, { selector: '.ribbon-tab' })).toBeInTheDocument();
    expect(screen.getByText(/Tools/i, { selector: '.ribbon-tab' })).toBeInTheDocument();
  });

  it('calls onOpenAppMenu when File tab is clicked', async () => {
    const user = userEvent.setup();
    const props = makeProps();
    renderWithProviders(<Ribbon {...props} />);
    await user.click(screen.getByText(/File/i, { selector: '.ribbon-tab.file-tab' }));
    expect(props.onOpenAppMenu).toHaveBeenCalledOnce();
  });

  describe('Start tab (default)', () => {
    it('shows New, Open, Save buttons', () => {
      renderWithProviders(<Ribbon {...makeProps()} />);
      expect(screen.getByTitle(/New/i)).toBeInTheDocument();
      expect(screen.getByTitle(/Open/i)).toBeInTheDocument();
      expect(screen.getByTitle(/Save/i)).toBeInTheDocument();
    });

    it('calls onNewProject when New button is clicked', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByTitle(/New/i));
      expect(props.onNewProject).toHaveBeenCalledOnce();
    });

    it('calls onCalculate when Calculate button is clicked', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByTitle(/Calculate/i));
      expect(props.onCalculate).toHaveBeenCalledOnce();
    });

    it('calls onOpenDialog with project-info when Project Info button is clicked', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByTitle(/Project info/i));
      expect(props.onOpenDialog).toHaveBeenCalledWith('project-info');
    });

    it('calls onTogglePreview when Preview button is clicked', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByTitle(/Preview/i));
      expect(props.onTogglePreview).toHaveBeenCalledOnce();
    });
  });

  describe('Tab switching', () => {
    it('switches to Envelope tab and shows zone/surface/construction buttons', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Building Envelope/i, { selector: '.ribbon-tab' }));
      expect(screen.getByTitle(/Add zone/i)).toBeInTheDocument();
      expect(screen.getByTitle(/Add surface/i)).toBeInTheDocument();
      expect(screen.getByTitle(/Add construction/i)).toBeInTheDocument();
    });

    it('switches to Installations tab and shows heating/ventilation/cooling buttons', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Installations/i, { selector: '.ribbon-tab' }));
      expect(screen.getByTitle(/Add heating/i)).toBeInTheDocument();
      expect(screen.getByTitle(/Add ventilation/i)).toBeInTheDocument();
      expect(screen.getByTitle(/Add cooling/i)).toBeInTheDocument();
    });

    it('switches to Renewables tab and shows solar buttons', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Renewables/i, { selector: '.ribbon-tab' }));
      expect(screen.getByTitle(/Add PV/i)).toBeInTheDocument();
      expect(screen.getByTitle(/Add Solar Thermal/i)).toBeInTheDocument();
    });

    it('switches to Report tab and shows export/print buttons', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Report/i, { selector: '.ribbon-tab' }));
      expect(screen.getByTitle(/Export Report/i)).toBeInTheDocument();
      expect(screen.getByTitle(/Print/i)).toBeInTheDocument();
    });
  });

  describe('Envelope tab dialog triggers', () => {
    it('calls onOpenDialog("zone-editor") when Add Zone is clicked', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Building Envelope/i, { selector: '.ribbon-tab' }));
      await user.click(screen.getByTitle(/Add zone/i));
      expect(props.onOpenDialog).toHaveBeenCalledWith('zone-editor');
    });

    it('calls onOpenDialog("surface-editor") when Add Surface is clicked', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Building Envelope/i, { selector: '.ribbon-tab' }));
      await user.click(screen.getByTitle(/Add surface/i));
      expect(props.onOpenDialog).toHaveBeenCalledWith('surface-editor');
    });

    it('calls onOpenDialog("window-editor") when Add Window is clicked', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Building Envelope/i, { selector: '.ribbon-tab' }));
      await user.click(screen.getByTitle(/Add window/i));
      expect(props.onOpenDialog).toHaveBeenCalledWith('window-editor');
    });

    it('calls onOpenDialog("thermal-bridge") when Add Thermal Bridge is clicked', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Building Envelope/i, { selector: '.ribbon-tab' }));
      await user.click(screen.getByTitle(/Add thermal bridge/i));
      expect(props.onOpenDialog).toHaveBeenCalledWith('thermal-bridge');
    });

    it('calls onOpenDialog("air-tightness") when Air Tightness is clicked', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Building Envelope/i, { selector: '.ribbon-tab' }));
      await user.click(screen.getByTitle(/Air tightness/i));
      expect(props.onOpenDialog).toHaveBeenCalledWith('air-tightness');
    });
  });

  describe('Installations tab dialog triggers', () => {
    it('calls onOpenDialog("heating-system")', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Installations/i, { selector: '.ribbon-tab' }));
      await user.click(screen.getByTitle(/Add heating/i));
      expect(props.onOpenDialog).toHaveBeenCalledWith('heating-system');
    });

    it('calls onOpenDialog("ventilation-system")', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Installations/i, { selector: '.ribbon-tab' }));
      await user.click(screen.getByTitle(/Add ventilation/i));
      expect(props.onOpenDialog).toHaveBeenCalledWith('ventilation-system');
    });

    it('calls onOpenDialog("cooling-system")', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Installations/i, { selector: '.ribbon-tab' }));
      await user.click(screen.getByTitle(/Add cooling/i));
      expect(props.onOpenDialog).toHaveBeenCalledWith('cooling-system');
    });

    it('calls onOpenDialog("hot-water-system")', async () => {
      const user = userEvent.setup();
      const props = makeProps();
      renderWithProviders(<Ribbon {...props} />);
      await user.click(screen.getByText(/Installations/i, { selector: '.ribbon-tab' }));
      await user.click(screen.getByTitle(/Add hot water/i));
      expect(props.onOpenDialog).toHaveBeenCalledWith('hot-water-system');
    });
  });
});
