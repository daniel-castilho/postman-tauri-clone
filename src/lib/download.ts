// src/lib/download.ts
//
// Shared browser download helper: wraps the payload in a Blob and triggers
// a detached anchor click. Used by every export surface (load test reports,
// collection runner reports).

export function downloadTextFile(fileName: string, content: string, mimeType: string): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = fileName;
  anchor.click();
  URL.revokeObjectURL(url);
}
