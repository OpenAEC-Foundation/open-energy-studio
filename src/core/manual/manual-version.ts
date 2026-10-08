/**
 * The kernel version the manual was reviewed for, from the stamp in its
 * index (BRL 9501 §4.4). Only the index is bundled here, so the settings
 * dialog can show the version without loading the whole manual.
 */
import indexText from '../../../docs/handleiding-nta8800/index.md?raw';
import { manualStamp } from '../../../scripts/nta-manual-markdown.mjs';

export const MANUAL_KERNEL_VERSION: string | null = manualStamp(indexText)?.kernelVersion ?? null;
