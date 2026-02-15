import type { ProcessorId } from '@wavecraft/core';
import { useHasProcessor } from '@wavecraft/core';

type UseHasProcessorArg = Parameters<typeof useHasProcessor>[0];

const validProcessorIdA: ProcessorId = 'example_processor';
const validProcessorIdB: ProcessorId = 'oscillator';
const validProcessorIdC: ProcessorId = 'oscilloscope_tap';

const validHookArg: UseHasProcessorArg = 'oscilloscope_tap';

void validProcessorIdA;
void validProcessorIdB;
void validProcessorIdC;
void validHookArg;

// @ts-expect-error invalid processor IDs should fail in augmented projects
const invalidProcessorId: ProcessorId = 'not_a_real_processor';

// @ts-expect-error useHasProcessor should reject invalid generated IDs in augmented projects
const invalidHookArg: UseHasProcessorArg = 'not_a_real_processor';

void invalidProcessorId;
void invalidHookArg;
