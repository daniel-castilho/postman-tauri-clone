import { describe, expect, it } from 'vitest';
import { classifyHttpStatus, isErrorStatus, isSuccessStatus } from '../lib/httpStatus';

describe('classifyHttpStatus', () => {
  it('maps informational, success, redirect, client and server families', () => {
    expect(classifyHttpStatus(100)).toBe('informational');
    expect(classifyHttpStatus(204)).toBe('success');
    expect(classifyHttpStatus(301)).toBe('redirect');
    expect(classifyHttpStatus(404)).toBe('clientError');
    expect(classifyHttpStatus(503)).toBe('serverError');
  });

  it('returns unknown for values outside 100–599', () => {
    expect(classifyHttpStatus(0)).toBe('unknown');
    expect(classifyHttpStatus(99)).toBe('unknown');
    expect(classifyHttpStatus(600)).toBe('unknown');
  });
});

describe('isSuccessStatus / isErrorStatus', () => {
  it('treats 2xx as success and 4xx/5xx as errors', () => {
    expect(isSuccessStatus(200)).toBe(true);
    expect(isSuccessStatus(404)).toBe(false);
    expect(isErrorStatus(400)).toBe(true);
    expect(isErrorStatus(500)).toBe(true);
    expect(isErrorStatus(201)).toBe(false);
  });
});
