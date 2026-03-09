import { useState, useRef, useCallback, useEffect } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import {
  ChevronRight, ChevronDown,
  Layers, Square, PanelTop, Thermometer,
  Wind, Snowflake, Droplets, Zap, SunMedium,
  Building2, Grid3X3, PanelLeftClose,
} from 'lucide-react';
import './ProjectBrowser.css';

interface TreeNodeProps {
  label: string;
  icon?: React.ReactNode;
  children?: React.ReactNode;
  selected?: boolean;
  onClick?: () => void;
  defaultOpen?: boolean;
}

function TreeNode({ label, icon, children, selected, onClick, defaultOpen = false }: TreeNodeProps) {
  const [open, setOpen] = useState(defaultOpen);
  const hasChildren = !!children;

  return (
    <div className="tree-node">
      <div
        className={`tree-node-header ${selected ? 'selected' : ''}`}
        onClick={() => {
          if (hasChildren) setOpen(!open);
          onClick?.();
        }}
      >
        {hasChildren ? (
          open ? <ChevronDown size={14} /> : <ChevronRight size={14} />
        ) : (
          <span style={{ width: 14 }} />
        )}
        {icon}
        <span className="tree-node-label">{label}</span>
      </div>
      {hasChildren && open && (
        <div className="tree-node-children">{children}</div>
      )}
    </div>
  );
}

export function ProjectBrowser() {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project, selectedItemId } = state;
  const [collapsed, setCollapsed] = useState(false);
  const [width, setWidth] = useState(260);
  const resizing = useRef(false);
  const startX = useRef(0);
  const startWidth = useRef(0);

  const onResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    resizing.current = true;
    startX.current = e.clientX;
    startWidth.current = width;
  }, [width]);

  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      if (!resizing.current) return;
      const newWidth = Math.min(500, Math.max(160, startWidth.current + (e.clientX - startX.current)));
      setWidth(newWidth);
    };
    const onMouseUp = () => { resizing.current = false; };
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
    return () => {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };
  }, []);

  const select = (id: string, type: string) => {
    dispatch({ type: 'SELECT_ITEM', payload: { id, itemType: type } });
  };

  if (collapsed) {
    return (
      <div className="project-browser collapsed" onClick={() => setCollapsed(false)}>
        <div className="panel-collapsed-label">
          <Building2 size={14} />
          <span>{t('browser.project')}</span>
        </div>
      </div>
    );
  }

  return (
    <div className="project-browser" style={{ width }}>
      <div className="project-browser-header">
        <Building2 size={14} />
        <span>{t('browser.project')}</span>
        <button className="panel-collapse-btn" onClick={() => setCollapsed(true)}><PanelLeftClose size={14} /></button>
      </div>
      <div className="project-browser-tree">
        {/* Zones */}
        <TreeNode
          label={t('browser.zones')}
          icon={<Layers size={14} />}
          defaultOpen={true}
        >
          {project.zones.map(zone => (
            <TreeNode
              key={zone.id}
              label={zone.name || t('browser.zone')}
              icon={<Layers size={14} />}
              selected={selectedItemId === zone.id}
              onClick={() => select(zone.id, 'zone')}
              defaultOpen={true}
            >
              {/* Surfaces */}
              <TreeNode label={t('browser.surfaces')} icon={<Square size={14} />}>
                {zone.surfaces.map(surface => (
                  <TreeNode
                    key={surface.id}
                    label={surface.name}
                    icon={<Square size={14} />}
                    selected={selectedItemId === surface.id}
                    onClick={() => select(surface.id, 'surface')}
                  >
                    {surface.windows.map(win => (
                      <TreeNode
                        key={win.id}
                        label={win.name}
                        icon={<PanelTop size={14} />}
                        selected={selectedItemId === win.id}
                        onClick={() => select(win.id, 'window')}
                      />
                    ))}
                  </TreeNode>
                ))}
              </TreeNode>
              {/* Thermal Bridges */}
              <TreeNode label={t('browser.thermalBridges')} icon={<Thermometer size={14} />}>
                {zone.thermalBridges.map(tb => (
                  <TreeNode
                    key={tb.id}
                    label={tb.name}
                    icon={<Thermometer size={14} />}
                    selected={selectedItemId === tb.id}
                    onClick={() => select(tb.id, 'thermalBridge')}
                  />
                ))}
              </TreeNode>
            </TreeNode>
          ))}
        </TreeNode>

        {/* Constructions */}
        <TreeNode
          label={t('browser.constructions')}
          icon={<Grid3X3 size={14} />}
        >
          {project.constructions.map(c => (
            <TreeNode
              key={c.id}
              label={c.name}
              icon={<Grid3X3 size={14} />}
              selected={selectedItemId === c.id}
              onClick={() => select(c.id, 'construction')}
            />
          ))}
        </TreeNode>

        {/* Installations */}
        <TreeNode
          label={t('browser.installations')}
          icon={<Thermometer size={14} />}
          defaultOpen={true}
        >
          <TreeNode label={t('browser.heating')} icon={<Thermometer size={14} />}>
            {project.heatingSystems.map(h => (
              <TreeNode
                key={h.id}
                label={h.name}
                selected={selectedItemId === h.id}
                onClick={() => select(h.id, 'heatingSystem')}
              />
            ))}
          </TreeNode>
          <TreeNode label={t('browser.ventilation')} icon={<Wind size={14} />}>
            {project.ventilationSystems.map(v => (
              <TreeNode
                key={v.id}
                label={v.name}
                selected={selectedItemId === v.id}
                onClick={() => select(v.id, 'ventilationSystem')}
              />
            ))}
          </TreeNode>
          <TreeNode label={t('browser.cooling')} icon={<Snowflake size={14} />}>
            {project.coolingSystems.map(c => (
              <TreeNode
                key={c.id}
                label={c.name}
                selected={selectedItemId === c.id}
                onClick={() => select(c.id, 'coolingSystem')}
              />
            ))}
          </TreeNode>
          <TreeNode label={t('browser.hotWater')} icon={<Droplets size={14} />}>
            {project.hotWaterSystems.map(hw => (
              <TreeNode
                key={hw.id}
                label={hw.name}
                selected={selectedItemId === hw.id}
                onClick={() => select(hw.id, 'hotWaterSystem')}
              />
            ))}
          </TreeNode>
        </TreeNode>

        {/* Renewables */}
        <TreeNode
          label={t('browser.renewables')}
          icon={<Zap size={14} />}
        >
          <TreeNode label={t('browser.solarPV')} icon={<Zap size={14} />}>
            {project.solarPV.map(pv => (
              <TreeNode
                key={pv.id}
                label={pv.name}
                selected={selectedItemId === pv.id}
                onClick={() => select(pv.id, 'solarPV')}
              />
            ))}
          </TreeNode>
          <TreeNode label={t('browser.solarThermal')} icon={<SunMedium size={14} />}>
            {project.solarThermal.map(st => (
              <TreeNode
                key={st.id}
                label={st.name}
                selected={selectedItemId === st.id}
                onClick={() => select(st.id, 'solarThermal')}
              />
            ))}
          </TreeNode>
        </TreeNode>
      </div>
      <div className="panel-resize-handle panel-resize-handle-right" onMouseDown={onResizeStart} />
    </div>
  );
}
