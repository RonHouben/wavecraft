/**
 * Global test setup
 *
 * This file runs before each test file.
 */

import '@testing-library/jest-dom';
import { beforeEach } from 'vitest';
import { resetMocks } from './mocks/ipc';

// Reset all mocks before each test
beforeEach(() => {
  resetMocks();
});
