import { useState, useRef, useEffect, useCallback } from 'react';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import {
  calculatePsiValue,
  createDefaultInput,
  DETAIL_TYPE_META,
  type ThermalBridgeInput,
  type ThermalBridgeResult,
  type DetailType,
  type InsulationPosition,
  type FrameMaterial,
} from '../../core/energy/ThermalBridgeCalc';
import './ThermalBridgeCalculator.css';

export function ThermalBridgeCalculator() {
  const { t, locale } = useI18n();
  const n = (value: number, digits: number) => formatNumber(value, locale, digits);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [input, setInput] = useState<ThermalBridgeInput>(createDefaultInput);
  const [result, setResult] = useState<ThermalBridgeResult | null>(null);

  // Recalculate whenever input changes
  useEffect(() => {
    const res = calculatePsiValue(input);
    setResult(res);
  }, [input]);

  // Draw the detail on canvas whenever input or result changes
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // HiDPI support
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    drawDetail(ctx, rect.width, rect.height, input, result, locale);
  }, [input, result, locale]);

  const meta = DETAIL_TYPE_META.find(m => m.id === input.detailType)!;

  const updateInput = useCallback((partial: Partial<ThermalBridgeInput>) => {
    setInput(prev => ({ ...prev, ...partial }));
  }, []);

  return (
    <div className="tb-calc">
      <div className="tb-calc-header">
        <h2>{t('tb.title')}</h2>
        <p className="tb-calc-subtitle">{t('tb.subtitle')}</p>
      </div>

      <div className="tb-calc-layout">
        {/* Left: Input panel */}
        <div className="tb-calc-inputs">
          <div className="tb-calc-section">
            <h3>{t('tb.detailType')}</h3>
            <div className="tb-detail-grid">
              {DETAIL_TYPE_META.map(m => (
                <button
                  key={m.id}
                  className={`tb-detail-btn ${input.detailType === m.id ? 'active' : ''}`}
                  onClick={() => updateInput({ detailType: m.id as DetailType })}
                >
                  <span className="tb-detail-icon">{getDetailIcon(m.id)}</span>
                  <span className="tb-detail-label">{t(m.labelKey)}</span>
                </button>
              ))}
            </div>
          </div>

          <div className="tb-calc-section">
            <h3>{t('tb.parameters')}</h3>

            <label className="tb-field">
              <span>{t('tb.rcWall')} (m2K/W)</span>
              <input
                type="number"
                min={0.5}
                max={10}
                step={0.1}
                value={input.rcWall}
                onChange={e => updateInput({ rcWall: parseFloat(e.target.value) || 0 })}
              />
            </label>

            {meta.needsRcFloorRoof && (
              <label className="tb-field">
                <span>
                  {meta.rcFloorRoofLabel === 'roof' ? t('tb.rcRoof') : t('tb.rcFloor')} (m2K/W)
                </span>
                <input
                  type="number"
                  min={0.5}
                  max={10}
                  step={0.1}
                  value={input.rcFloorRoof}
                  onChange={e => updateInput({ rcFloorRoof: parseFloat(e.target.value) || 0 })}
                />
              </label>
            )}

            {meta.needsInsulationPos && (
              <label className="tb-field">
                <span>{t('tb.insulationPosition')}</span>
                <select
                  value={input.insulationPosition}
                  onChange={e => updateInput({ insulationPosition: e.target.value as InsulationPosition })}
                >
                  <option value="inside">{t('tb.posInside')}</option>
                  <option value="cavity">{t('tb.posCavity')}</option>
                  <option value="outside">{t('tb.posOutside')}</option>
                </select>
              </label>
            )}

            {meta.needsFrameMaterial && (
              <label className="tb-field">
                <span>{t('tb.frameMaterial')}</span>
                <select
                  value={input.frameMaterial}
                  onChange={e => updateInput({ frameMaterial: e.target.value as FrameMaterial })}
                >
                  <option value="wood">{t('tb.frameWood')}</option>
                  <option value="plastic">{t('tb.framePlastic')}</option>
                  <option value="aluminium">{t('tb.frameAluminium')}</option>
                </select>
              </label>
            )}

            {meta.needsFrameWidth && (
              <label className="tb-field">
                <span>{t('tb.frameWidth')} (mm)</span>
                <input
                  type="number"
                  min={40}
                  max={120}
                  step={5}
                  value={input.frameWidth}
                  onChange={e => updateInput({ frameWidth: parseFloat(e.target.value) || 60 })}
                />
              </label>
            )}

            {meta.needsFoundationDepth && (
              <label className="tb-field">
                <span>{t('tb.foundationDepth')} (m)</span>
                <input
                  type="number"
                  min={0.3}
                  max={2.0}
                  step={0.1}
                  value={input.foundationDepth}
                  onChange={e => updateInput({ foundationDepth: parseFloat(e.target.value) || 0.8 })}
                />
              </label>
            )}

            {meta.needsThermalBreak && (
              <label className="tb-field tb-field-checkbox">
                <input
                  type="checkbox"
                  checked={input.hasThermalBreak}
                  onChange={e => updateInput({ hasThermalBreak: e.target.checked })}
                />
                <span>{t('tb.thermalBreak')}</span>
              </label>
            )}
          </div>
        </div>

        {/* Center: Canvas drawing */}
        <div className="tb-calc-canvas-wrap">
          <canvas ref={canvasRef} className="tb-calc-canvas" />
          <div className="tb-canvas-legend">
            <span className="tb-legend-item"><span className="tb-legend-swatch" style={{ background: '#b91c1c' }} /> {t('tb.legendBrick')}</span>
            <span className="tb-legend-item"><span className="tb-legend-swatch" style={{ background: '#9ca3af' }} /> {t('tb.legendConcrete')}</span>
            <span className="tb-legend-item"><span className="tb-legend-swatch" style={{ background: '#facc15' }} /> {t('tb.legendInsulation')}</span>
            <span className="tb-legend-item"><span className="tb-legend-swatch" style={{ background: '#92400e' }} /> {t('tb.legendWood')}</span>
          </div>
        </div>

        {/* Right: Results */}
        <div className="tb-calc-results">
          {result && (
            <>
              <div className="tb-result-card tb-result-main">
                <div className="tb-result-label">{t('tb.psiCalculated')}</div>
                <div className="tb-result-value">
                  {n(result.psiCalculated, 3)}
                  <span className="tb-result-unit">W/(mK)</span>
                </div>
              </div>

              <div className="tb-result-card">
                <div className="tb-result-label">{t('tb.psiForfait')}</div>
                <div className="tb-result-value tb-result-forfait">
                  {n(result.psiForfait, 3)}
                  <span className="tb-result-unit">W/(mK)</span>
                </div>
              </div>

              <div className="tb-result-card">
                <div className="tb-result-label">{t('tb.percentForfait')}</div>
                <div className={`tb-result-value ${result.percentOfForfait <= 100 ? 'tb-result-good' : 'tb-result-bad'}`}>
                  {n(result.percentOfForfait, 0)}%
                </div>
              </div>

              <div className="tb-result-card">
                <div className="tb-result-label">{t('tb.heatLoss')}</div>
                <div className="tb-result-value">
                  {n(result.heatLossPerMeter, 3)}
                  <span className="tb-result-unit">W/m</span>
                </div>
                <div className="tb-result-note">{t('tb.heatLossNote')}</div>
              </div>

              <div className="tb-result-comparison">
                <div className="tb-comparison-bar-wrap">
                  <div className="tb-comparison-label">{t('tb.calculated')}</div>
                  <div className="tb-comparison-bar-bg">
                    <div
                      className="tb-comparison-bar tb-comparison-bar-calc"
                      style={{ width: `${Math.min(100, (result.psiCalculated / result.psiForfait) * 100)}%` }}
                    />
                  </div>
                </div>
                <div className="tb-comparison-bar-wrap">
                  <div className="tb-comparison-label">{t('tb.forfait')}</div>
                  <div className="tb-comparison-bar-bg">
                    <div
                      className="tb-comparison-bar tb-comparison-bar-forfait"
                      style={{ width: '100%' }}
                    />
                  </div>
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

// ============================================================
// Detail type icons (simple SVG-like symbols via unicode)
// ============================================================

function getDetailIcon(type: DetailType): string {
  const icons: Record<DetailType, string> = {
    'wall-floor':         '\u2534',  // bottom T
    'wall-roof':          '\u252C',  // top T
    'wall-foundation':    '\u2568',  // foundation symbol
    'window-frame':       '\u25A1',  // square
    'wall-corner':        '\u2514',  // corner
    'wall-internal-wall': '\u251C',  // left T
    'roof-internal-wall': '\u2564',  // T with lines
    'balcony':            '\u2510',  // right angle
  };
  return icons[type] || '\u25A0';
}

// ============================================================
// Canvas 2D Detail Drawing
// ============================================================

function drawDetail(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  input: ThermalBridgeInput,
  result: ThermalBridgeResult | null,
  locale: string,
) {
  // Clear
  ctx.clearRect(0, 0, w, h);

  // Background
  ctx.fillStyle = '#0f1419';
  ctx.fillRect(0, 0, w, h);

  // Grid
  drawGrid(ctx, w, h);

  // Center point
  const cx = w / 2;
  const cy = h / 2;

  switch (input.detailType) {
    case 'wall-floor':
      drawWallFloorDetail(ctx, cx, cy, w, h, input);
      break;
    case 'wall-roof':
      drawWallRoofDetail(ctx, cx, cy, w, h, input);
      break;
    case 'wall-foundation':
      drawWallFoundationDetail(ctx, cx, cy, w, h, input);
      break;
    case 'window-frame':
      drawWindowFrameDetail(ctx, cx, cy, w, h, input);
      break;
    case 'wall-corner':
      drawWallCornerDetail(ctx, cx, cy, w, h, input);
      break;
    case 'wall-internal-wall':
      drawWallInternalWallDetail(ctx, cx, cy, w, h, input);
      break;
    case 'roof-internal-wall':
      drawRoofInternalWallDetail(ctx, cx, cy, w, h, input);
      break;
    case 'balcony':
      drawBalconyDetail(ctx, cx, cy, w, h, input);
      break;
  }

  // Draw heat flow arrows
  if (result) {
    drawHeatFlowArrows(ctx, cx, cy, w, h, input, result);
  }

  // Draw psi annotation
  if (result) {
    drawPsiAnnotation(ctx, w, h, result, locale);
  }
}

function drawGrid(ctx: CanvasRenderingContext2D, w: number, h: number) {
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.04)';
  ctx.lineWidth = 0.5;
  const step = 20;
  for (let x = 0; x < w; x += step) {
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, h);
    ctx.stroke();
  }
  for (let y = 0; y < h; y += step) {
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(w, y);
    ctx.stroke();
  }
}

// ---- Material colors ----
const COLORS = {
  brick: '#b91c1c',
  brickDark: '#7f1d1d',
  concrete: '#9ca3af',
  concreteDark: '#6b7280',
  insulation: '#facc15',
  insulationDark: '#ca8a04',
  wood: '#92400e',
  woodLight: '#b45309',
  steel: '#475569',
  steelDark: '#334155',
  ground: '#78716c',
  groundDark: '#57534e',
  interior: '#dbeafe',
  exterior: '#bfdbfe',
};

// ---- Helper drawing functions ----

function drawBrickPattern(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number) {
  ctx.fillStyle = COLORS.brick;
  ctx.fillRect(x, y, w, h);
  ctx.strokeStyle = COLORS.brickDark;
  ctx.lineWidth = 0.8;
  const brickH = 8;
  const brickW = 16;
  for (let row = 0; row < h / brickH; row++) {
    const yy = y + row * brickH;
    ctx.beginPath();
    ctx.moveTo(x, yy);
    ctx.lineTo(x + w, yy);
    ctx.stroke();
    const offset = row % 2 === 0 ? 0 : brickW / 2;
    for (let col = 0; col < w / brickW + 1; col++) {
      const xx = x + col * brickW + offset;
      if (xx > x && xx < x + w) {
        ctx.beginPath();
        ctx.moveTo(xx, yy);
        ctx.lineTo(xx, yy + brickH);
        ctx.stroke();
      }
    }
  }
}

function drawConcretePattern(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number) {
  ctx.fillStyle = COLORS.concrete;
  ctx.fillRect(x, y, w, h);
  // Stipple pattern
  ctx.fillStyle = COLORS.concreteDark;
  for (let i = 0; i < (w * h) / 80; i++) {
    const px = x + Math.random() * w;
    const py = y + Math.random() * h;
    ctx.fillRect(px, py, 1.5, 1.5);
  }
}

function drawInsulationPattern(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number) {
  ctx.fillStyle = COLORS.insulation;
  ctx.fillRect(x, y, w, h);
  // Zigzag hatching for insulation
  ctx.strokeStyle = COLORS.insulationDark;
  ctx.lineWidth = 0.8;
  const zigStep = 12;
  const amp = Math.min(w, h) * 0.3;
  if (w > h) {
    // Horizontal insulation
    for (let yy = y + 4; yy < y + h - 4; yy += zigStep) {
      ctx.beginPath();
      for (let xx = x; xx < x + w; xx += 6) {
        const offset = ((xx - x) % 12 < 6) ? -amp : amp;
        ctx.lineTo(xx, yy + offset * 0.3);
      }
      ctx.stroke();
    }
  } else {
    // Vertical insulation
    for (let xx = x + 4; xx < x + w - 4; xx += zigStep) {
      ctx.beginPath();
      for (let yy = y; yy < y + h; yy += 6) {
        const offset = ((yy - y) % 12 < 6) ? -amp : amp;
        ctx.lineTo(xx + offset * 0.3, yy);
      }
      ctx.stroke();
    }
  }
}

function drawWoodPattern(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number) {
  ctx.fillStyle = COLORS.wood;
  ctx.fillRect(x, y, w, h);
  ctx.strokeStyle = COLORS.woodLight;
  ctx.lineWidth = 0.5;
  for (let i = 0; i < 5; i++) {
    const yy = y + Math.random() * h;
    ctx.beginPath();
    ctx.moveTo(x, yy);
    ctx.bezierCurveTo(x + w * 0.3, yy - 3, x + w * 0.7, yy + 3, x + w, yy);
    ctx.stroke();
  }
}

function drawGroundPattern(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number) {
  ctx.fillStyle = COLORS.ground;
  ctx.fillRect(x, y, w, h);
  // Dots for ground
  ctx.fillStyle = COLORS.groundDark;
  for (let i = 0; i < (w * h) / 50; i++) {
    const px = x + Math.random() * w;
    const py = y + Math.random() * h;
    ctx.beginPath();
    ctx.arc(px, py, 1, 0, Math.PI * 2);
    ctx.fill();
  }
}

function drawDimensionLine(
  ctx: CanvasRenderingContext2D,
  x1: number, y1: number, x2: number, y2: number,
  label: string, offset: number = 15
) {
  const isHorizontal = Math.abs(y2 - y1) < Math.abs(x2 - x1);
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.6)';
  ctx.lineWidth = 0.8;
  ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
  ctx.font = '10px monospace';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';

  if (isHorizontal) {
    const dy = offset;
    // Extension lines
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x1, y1 + dy);
    ctx.moveTo(x2, y2);
    ctx.lineTo(x2, y2 + dy);
    // Dimension line
    ctx.moveTo(x1, y1 + dy * 0.7);
    ctx.lineTo(x2, y2 + dy * 0.7);
    ctx.stroke();
    // Arrowheads
    drawArrowhead(ctx, x1, y1 + dy * 0.7, 0);
    drawArrowhead(ctx, x2, y2 + dy * 0.7, Math.PI);
    // Label
    ctx.fillText(label, (x1 + x2) / 2, y1 + dy * 0.7 - 8);
  } else {
    const dx = offset;
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x1 + dx, y1);
    ctx.moveTo(x2, y2);
    ctx.lineTo(x2 + dx, y2);
    ctx.moveTo(x1 + dx * 0.7, y1);
    ctx.lineTo(x2 + dx * 0.7, y2);
    ctx.stroke();
    drawArrowhead(ctx, x1 + dx * 0.7, y1, -Math.PI / 2);
    drawArrowhead(ctx, x2 + dx * 0.7, y2, Math.PI / 2);
    ctx.save();
    ctx.translate(x1 + dx * 0.7 + 10, (y1 + y2) / 2);
    ctx.rotate(-Math.PI / 2);
    ctx.fillText(label, 0, 0);
    ctx.restore();
  }
}

function drawArrowhead(ctx: CanvasRenderingContext2D, x: number, y: number, angle: number) {
  const size = 5;
  ctx.save();
  ctx.translate(x, y);
  ctx.rotate(angle);
  ctx.beginPath();
  ctx.moveTo(0, 0);
  ctx.lineTo(-size, -size * 0.4);
  ctx.lineTo(-size, size * 0.4);
  ctx.closePath();
  ctx.fillStyle = 'rgba(255, 255, 255, 0.6)';
  ctx.fill();
  ctx.restore();
}

function drawLabel(ctx: CanvasRenderingContext2D, x: number, y: number, text: string) {
  ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
  ctx.font = '9px sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, x, y);
}

// ---- Detail-specific drawings ----

function getInsulationX(cx: number, wallW: number, pos: InsulationPosition, insW: number): number {
  switch (pos) {
    case 'inside': return cx - wallW / 2;
    case 'outside': return cx + wallW / 2 - insW;
    case 'cavity': return cx - insW / 2;
  }
}

function drawWallFloorDetail(ctx: CanvasRenderingContext2D, cx: number, cy: number, w: number, h: number, input: ThermalBridgeInput) {
  const wallW = 60;
  const floorH = 40;
  const insW = 20;
  const wallLeft = cx - wallW / 2;

  // Floor slab (concrete)
  drawConcretePattern(ctx, cx - w * 0.4, cy, w * 0.8, floorH);
  drawLabel(ctx, cx + 60, cy + floorH / 2, `Rc=${input.rcFloorRoof}`);

  // Floor insulation (on top or bottom)
  drawInsulationPattern(ctx, cx - w * 0.4, cy - insW, w * 0.8, insW);

  // Wall (brick outer, insulation, inner leaf)
  const wallTop = cy - h * 0.35;
  const wallHeight = cy - wallTop;

  // Outer leaf (brick)
  drawBrickPattern(ctx, wallLeft, wallTop, (wallW - insW) / 2, wallHeight);

  // Insulation
  const insX = getInsulationX(cx, wallW, input.insulationPosition, insW);
  drawInsulationPattern(ctx, insX, wallTop, insW, wallHeight);

  // Inner leaf (concrete block)
  const innerX = wallLeft + (wallW - insW) / 2 + insW;
  const innerW = wallLeft + wallW - innerX;
  drawConcretePattern(ctx, innerX, wallTop, innerW, wallHeight);

  // Below floor: continuation of wall
  drawBrickPattern(ctx, wallLeft, cy + floorH, wallW, h * 0.15);

  // Labels
  drawLabel(ctx, cx, wallTop + 15, `Rc=${input.rcWall}`);

  // Dimension lines
  drawDimensionLine(ctx, wallLeft, cy + floorH + h * 0.15 + 5, wallLeft + wallW, cy + floorH + h * 0.15 + 5, `${wallW * 5}mm`, 20);

  // Interior / Exterior labels
  drawLabel(ctx, wallLeft - 30, cy - 20, 'EXT');
  drawLabel(ctx, wallLeft + wallW + 30, cy - 20, 'INT');
}

function drawWallRoofDetail(ctx: CanvasRenderingContext2D, cx: number, cy: number, w: number, h: number, input: ThermalBridgeInput) {
  const wallW = 60;
  const roofH = 30;
  const insW = 20;
  const wallLeft = cx - wallW / 2;

  // Wall below
  const wallBottom = cy + h * 0.35;
  const wallHeight = wallBottom - cy;

  drawBrickPattern(ctx, wallLeft, cy, (wallW - insW) / 2, wallHeight);
  const insX = getInsulationX(cx, wallW, input.insulationPosition, insW);
  drawInsulationPattern(ctx, insX, cy, insW, wallHeight);
  const innerX = wallLeft + (wallW - insW) / 2 + insW;
  drawConcretePattern(ctx, innerX, cy, wallLeft + wallW - innerX, wallHeight);

  // Roof structure
  drawConcretePattern(ctx, cx - w * 0.35, cy - roofH, w * 0.7, roofH);

  // Roof insulation on top
  drawInsulationPattern(ctx, cx - w * 0.35, cy - roofH - insW, w * 0.7, insW);

  drawLabel(ctx, cx, wallBottom - 20, `Rc=${input.rcWall}`);
  drawLabel(ctx, cx + 60, cy - roofH / 2, `Rc=${input.rcFloorRoof}`);

  drawLabel(ctx, wallLeft - 30, cy + 30, 'EXT');
  drawLabel(ctx, wallLeft + wallW + 30, cy + 30, 'INT');
}

function drawWallFoundationDetail(ctx: CanvasRenderingContext2D, cx: number, cy: number, w: number, h: number, input: ThermalBridgeInput) {
  const wallW = 60;
  const insW = 20;
  const wallLeft = cx - wallW / 2;
  const groundLevel = cy;
  const foundDepthPx = Math.min(input.foundationDepth * 80, h * 0.35);

  // Ground
  drawGroundPattern(ctx, 0, groundLevel, w, h - groundLevel);

  // Foundation (concrete) below ground
  drawConcretePattern(ctx, wallLeft - 15, groundLevel, wallW + 30, foundDepthPx);

  // Wall above ground
  const wallTop = cy - h * 0.3;
  drawBrickPattern(ctx, wallLeft, wallTop, (wallW - insW) / 2, groundLevel - wallTop);
  const insX = getInsulationX(cx, wallW, input.insulationPosition, insW);
  drawInsulationPattern(ctx, insX, wallTop, insW, groundLevel - wallTop);
  const innerX = wallLeft + (wallW - insW) / 2 + insW;
  drawConcretePattern(ctx, innerX, wallTop, wallLeft + wallW - innerX, groundLevel - wallTop);

  // Floor slab at ground
  drawConcretePattern(ctx, cx - 10, groundLevel, w * 0.35, 25);
  drawInsulationPattern(ctx, cx - 10, groundLevel - insW, w * 0.35, insW);

  // Ground level line
  ctx.strokeStyle = '#22c55e';
  ctx.lineWidth = 1.5;
  ctx.setLineDash([6, 4]);
  ctx.beginPath();
  ctx.moveTo(0, groundLevel);
  ctx.lineTo(w, groundLevel);
  ctx.stroke();
  ctx.setLineDash([]);

  drawLabel(ctx, cx, wallTop + 15, `Rc=${input.rcWall}`);

  // Foundation depth dimension
  drawDimensionLine(ctx, wallLeft - 25, groundLevel, wallLeft - 25, groundLevel + foundDepthPx, `${input.foundationDepth}m`, -20);

  drawLabel(ctx, wallLeft - 40, wallTop + 20, 'EXT');
  drawLabel(ctx, wallLeft + wallW + 40, wallTop + 20, 'INT');
}

function drawWindowFrameDetail(ctx: CanvasRenderingContext2D, cx: number, cy: number, w: number, h: number, input: ThermalBridgeInput) {
  const wallW = 60;
  const insW = 20;
  const wallLeft = cx - wallW / 2 - 40;
  const frameW = Math.max(15, input.frameWidth / 4);

  // Wall section (left part)
  drawBrickPattern(ctx, wallLeft, cy - h * 0.35, (wallW - insW) / 2, h * 0.7);
  drawInsulationPattern(ctx, wallLeft + (wallW - insW) / 2, cy - h * 0.35, insW, h * 0.7);
  const innerX = wallLeft + (wallW - insW) / 2 + insW;
  drawConcretePattern(ctx, innerX, cy - h * 0.35, wallLeft + wallW - innerX, h * 0.7);

  // Window frame
  const frameX = wallLeft + wallW;

  if (input.frameMaterial === 'wood') {
    drawWoodPattern(ctx, frameX, cy - h * 0.35, frameW, h * 0.7);
  } else {
    const frameColor = input.frameMaterial === 'aluminium' ? COLORS.steel : '#e5e7eb';
    ctx.fillStyle = frameColor;
    ctx.fillRect(frameX, cy - h * 0.35, frameW, h * 0.7);
  }
  ctx.strokeStyle = input.frameMaterial === 'aluminium' ? COLORS.steelDark : COLORS.woodLight;
  ctx.lineWidth = 1;
  ctx.strokeRect(frameX, cy - h * 0.35, frameW, h * 0.7);

  // Glass pane
  const glassX = frameX + frameW;
  const glassW = w * 0.25;
  ctx.fillStyle = 'rgba(147, 197, 253, 0.3)';
  ctx.fillRect(glassX, cy - h * 0.3, glassW, h * 0.6);
  ctx.strokeStyle = 'rgba(147, 197, 253, 0.6)';
  ctx.lineWidth = 1.5;
  ctx.strokeRect(glassX, cy - h * 0.3, glassW, h * 0.6);

  // Double glazing lines
  ctx.strokeStyle = 'rgba(147, 197, 253, 0.4)';
  ctx.lineWidth = 0.5;
  ctx.beginPath();
  ctx.moveTo(glassX + 4, cy - h * 0.3);
  ctx.lineTo(glassX + 4, cy + h * 0.3);
  ctx.moveTo(glassX + glassW - 4, cy - h * 0.3);
  ctx.lineTo(glassX + glassW - 4, cy + h * 0.3);
  ctx.stroke();

  drawLabel(ctx, wallLeft + wallW / 2, cy - h * 0.35 - 10, `Rc=${input.rcWall}`);
  drawLabel(ctx, frameX + frameW / 2, cy + h * 0.35 + 12, `${input.frameWidth}mm`);
  drawLabel(ctx, glassX + glassW / 2, cy, input.frameMaterial);

  drawLabel(ctx, wallLeft - 25, cy, 'EXT');
  drawLabel(ctx, glassX + glassW + 25, cy, 'INT');
}

function drawWallCornerDetail(ctx: CanvasRenderingContext2D, cx: number, cy: number, w: number, h: number, input: ThermalBridgeInput) {
  const wallW = 50;
  const insW = 16;

  // Horizontal wall (going right from corner)
  const hWallY = cy;
  drawBrickPattern(ctx, cx, hWallY, w * 0.35, (wallW - insW) / 2);
  drawInsulationPattern(ctx, cx, hWallY + (wallW - insW) / 2, w * 0.35, insW);
  drawConcretePattern(ctx, cx, hWallY + (wallW - insW) / 2 + insW, w * 0.35, (wallW - insW) / 2);

  // Vertical wall (going up from corner)
  drawBrickPattern(ctx, cx - wallW, cy - h * 0.35, (wallW - insW) / 2, h * 0.35);
  drawInsulationPattern(ctx, cx - wallW + (wallW - insW) / 2, cy - h * 0.35, insW, h * 0.35);
  drawConcretePattern(ctx, cx - wallW + (wallW - insW) / 2 + insW, cy - h * 0.35, (wallW - insW) / 2, h * 0.35);

  // Corner block
  drawConcretePattern(ctx, cx - wallW, cy, wallW, wallW);

  drawLabel(ctx, cx + w * 0.2, hWallY - 10, `Rc=${input.rcWall}`);

  drawLabel(ctx, cx - wallW - 25, cy - h * 0.2, 'EXT');
  drawLabel(ctx, cx + 20, cy + wallW + 15, 'INT');
}

function drawWallInternalWallDetail(ctx: CanvasRenderingContext2D, cx: number, cy: number, w: number, h: number, input: ThermalBridgeInput) {
  const wallW = 60;
  const insW = 20;
  const intWallW = 30;
  const wallLeft = cx - wallW / 2;

  // Main wall (full height)
  drawBrickPattern(ctx, wallLeft, cy - h * 0.35, (wallW - insW) / 2, h * 0.7);
  drawInsulationPattern(ctx, wallLeft + (wallW - insW) / 2, cy - h * 0.35, insW, h * 0.7);
  drawConcretePattern(ctx, wallLeft + (wallW - insW) / 2 + insW, cy - h * 0.35, (wallW - insW) / 2, h * 0.7);

  // Internal wall (perpendicular, to the right)
  const intWallX = wallLeft + wallW;
  drawConcretePattern(ctx, intWallX, cy - intWallW / 2, w * 0.25, intWallW);

  drawLabel(ctx, cx, cy - h * 0.35 - 10, `Rc=${input.rcWall}`);

  drawLabel(ctx, wallLeft - 25, cy, 'EXT');
  drawLabel(ctx, intWallX + w * 0.15, cy - intWallW / 2 - 10, 'INT');
}

function drawRoofInternalWallDetail(ctx: CanvasRenderingContext2D, cx: number, cy: number, w: number, h: number, input: ThermalBridgeInput) {
  const roofH = 30;
  const insW = 20;
  const intWallW = 30;

  // Roof slab
  drawConcretePattern(ctx, cx - w * 0.35, cy, w * 0.7, roofH);

  // Roof insulation on top
  drawInsulationPattern(ctx, cx - w * 0.35, cy - insW, w * 0.7, insW);

  // Internal wall below roof
  drawConcretePattern(ctx, cx - intWallW / 2, cy + roofH, intWallW, h * 0.25);

  drawLabel(ctx, cx + w * 0.2, cy + roofH / 2, `Rc=${input.rcFloorRoof}`);

  drawLabel(ctx, cx, cy - insW - 10, 'EXT');
  drawLabel(ctx, cx, cy + roofH + h * 0.15, 'INT');
}

function drawBalconyDetail(ctx: CanvasRenderingContext2D, cx: number, cy: number, w: number, h: number, input: ThermalBridgeInput) {
  const wallW = 60;
  const insW = 20;
  const slabH = 30;
  const wallLeft = cx - wallW / 2;

  // Wall
  drawBrickPattern(ctx, wallLeft, cy - h * 0.35, (wallW - insW) / 2, h * 0.7);
  drawInsulationPattern(ctx, wallLeft + (wallW - insW) / 2, cy - h * 0.35, insW, h * 0.7);
  drawConcretePattern(ctx, wallLeft + (wallW - insW) / 2 + insW, cy - h * 0.35, (wallW - insW) / 2, h * 0.7);

  // Floor slab going through wall (interior)
  drawConcretePattern(ctx, wallLeft + wallW, cy - slabH / 2, w * 0.25, slabH);

  // Balcony slab (exterior — penetrating through)
  drawConcretePattern(ctx, wallLeft - w * 0.25, cy - slabH / 2, w * 0.25, slabH);

  // Thermal break (if present)
  if (input.hasThermalBreak) {
    ctx.fillStyle = '#f97316';
    ctx.fillRect(wallLeft - 3, cy - slabH / 2, 6, slabH);
    drawLabel(ctx, wallLeft - 3, cy - slabH / 2 - 10, 'TB');
  }

  drawLabel(ctx, cx, cy - h * 0.35 - 10, `Rc=${input.rcWall}`);
  drawLabel(ctx, wallLeft - w * 0.15, cy - slabH / 2 - 12, 'Balkon');

  drawLabel(ctx, wallLeft - w * 0.15, cy + 30, 'EXT');
  drawLabel(ctx, wallLeft + wallW + w * 0.15, cy + 30, 'INT');
}

// ---- Heat flow arrows ----

function drawHeatFlowArrows(
  ctx: CanvasRenderingContext2D,
  cx: number, cy: number,
  _w: number, _h: number,
  _input: ThermalBridgeInput,
  result: ThermalBridgeResult
) {
  const intensity = Math.min(1, result.psiCalculated / 0.5);
  const arrowCount = Math.max(3, Math.round(intensity * 8));

  ctx.save();
  ctx.globalAlpha = 0.5 + intensity * 0.3;

  for (let i = 0; i < arrowCount; i++) {
    const spread = 30;
    const offsetY = (i - arrowCount / 2) * (spread / arrowCount);
    const offsetX = (i - arrowCount / 2) * (spread / arrowCount) * 0.5;

    // Gradient from warm (red) to cool (blue)
    const t = i / Math.max(1, arrowCount - 1);
    const r = Math.round(239 - t * 120);
    const g = Math.round(68 + t * 60);
    const b = Math.round(68 + t * 150);
    ctx.strokeStyle = `rgb(${r},${g},${b})`;
    ctx.lineWidth = 1.5;

    // Arrow from interior toward exterior through junction
    const startX = cx + 50 + offsetX;
    const startY = cy + offsetY;
    const endX = cx - 50 + offsetX;
    const endY = cy + offsetY;

    ctx.beginPath();
    ctx.moveTo(startX, startY);
    ctx.lineTo(endX, endY);
    ctx.stroke();

    // Arrowhead
    drawArrowhead(ctx, endX, endY, Math.PI);
  }

  ctx.restore();
}

// ---- Psi annotation ----

function drawPsiAnnotation(ctx: CanvasRenderingContext2D, w: number, _h: number, result: ThermalBridgeResult, locale: string) {
  const padding = 10;
  const boxW = 140;
  const boxH = 28;
  const x = w - boxW - padding;
  const y = padding;

  ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
  ctx.beginPath();
  ctx.roundRect(x, y, boxW, boxH, 4);
  ctx.fill();

  ctx.fillStyle = '#facc15';
  ctx.font = 'bold 12px monospace';
  ctx.textAlign = 'left';
  ctx.textBaseline = 'middle';
  ctx.fillText(`\u03C8 = ${formatNumber(result.psiCalculated, locale, 3)} W/(mK)`, x + 8, y + boxH / 2);
}
