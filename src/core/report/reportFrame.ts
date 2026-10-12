/**
 * Sandbox of the iframes that show report HTML (report builder, print
 * preview). Report HTML is built from project text; it is escaped, but a
 * frame that shares the app's origin and may run scripts could reach the
 * Tauri IPC if one string ever slipped through. So the frames run no
 * scripts at all: `allow-same-origin` only lets this app measure and print
 * the frame's document. No `allow-scripts`, ever: together with
 * `allow-same-origin` it would let the frame remove its own sandbox.
 */
export const REPORT_FRAME_SANDBOX = 'allow-same-origin';

/** The hidden frame used for printing also needs the print dialog. */
export const REPORT_PRINT_SANDBOX = 'allow-same-origin allow-modals';
