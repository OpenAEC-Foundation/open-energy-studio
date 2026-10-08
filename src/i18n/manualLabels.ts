/**
 * Labels of the in-app user manual (Gereedschap › Handleiding, BRL 9501 §4.4).
 * Kept in their own file so parallel work on nl.ts/en.ts merges cleanly.
 */
export const manualLabelsNl: Record<string, string> = {
  'manual.title': 'Handleiding',
  'manual.lead': 'De gebruikershandleiding van deze versie van het programma. Ze hoort bij elke release en draagt dezelfde versie.',
  'manual.contents': 'Inhoud van de handleiding',
  'manual.programVersion': 'Programmaversie',
  'manual.manualKernel': 'Handleiding bij rekenkern',
  'manual.runningKernel': 'Rekenkern van deze installatie',
  'manual.kernelMismatch': 'De handleiding is nagelopen voor rekenkern {manual}, maar dit programma rekent met rekenkern {kernel}.',
  'manual.documentRef': 'Zie {path} in de documentatie bij het programma.',
  'manual.openChapter': 'Het hoofdstuk van de handleiding over deze stap',
  'manual.openManual': 'Handleiding openen',
};

export const manualLabelsEn: Record<string, string> = {
  'manual.title': 'Manual',
  'manual.lead': 'The user manual of this version of the program. It ships with every release and carries the same version.',
  'manual.contents': 'Manual contents',
  'manual.programVersion': 'Program version',
  'manual.manualKernel': 'Manual for calculation core',
  'manual.runningKernel': 'Calculation core of this installation',
  'manual.kernelMismatch': 'The manual was reviewed for calculation core {manual}, but this program calculates with core {kernel}.',
  'manual.documentRef': 'See {path} in the documentation that comes with the program.',
  'manual.openChapter': 'The manual chapter about this step',
  'manual.openManual': 'Open manual',
};
