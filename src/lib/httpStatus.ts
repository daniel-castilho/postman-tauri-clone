export type HttpStatusClass =
  'informational' | 'success' | 'redirect' | 'clientError' | 'serverError' | 'unknown';

/** Classify an HTTP status code into the standard 1xx–5xx families. */
export function classifyHttpStatus(status: number): HttpStatusClass {
  if (status >= 100 && status < 200) return 'informational';
  if (status >= 200 && status < 300) return 'success';
  if (status >= 300 && status < 400) return 'redirect';
  if (status >= 400 && status < 500) return 'clientError';
  if (status >= 500 && status < 600) return 'serverError';
  return 'unknown';
}

export function isSuccessStatus(status: number): boolean {
  return classifyHttpStatus(status) === 'success';
}

export function isErrorStatus(status: number): boolean {
  const klass = classifyHttpStatus(status);
  return klass === 'clientError' || klass === 'serverError';
}
