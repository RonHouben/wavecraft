import type { ProcessorId } from '@wavecraft/core';
import { useHasProcessorInSignalChain } from '@wavecraft/core';

type UseHasProcessorArg = Parameters<typeof useHasProcessorInSignalChain>[0];
type IsWidenedString = string extends UseHasProcessorArg ? true : false;

const validProcessorId: ProcessorId = 'oscillator';
const validHookArg: UseHasProcessorArg = 'oscillator';
const shouldBeLiteralUnion: IsWidenedString = false;

void validProcessorId;
void validHookArg;
void shouldBeLiteralUnion;

// @ts-expect-error invalid processor IDs should fail when generated augmentation is visible
const invalidProcessorId: ProcessorId = 'not_a_real_processor';

// @ts-expect-error useHasProcessorInSignalChain should reject invalid generated IDs when augmentation is visible
const invalidHookArg: UseHasProcessorArg = 'not_a_real_processor';

void invalidProcessorId;
void invalidHookArg;
