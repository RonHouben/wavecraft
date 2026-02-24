// eslint-disable-next-line no-restricted-imports -- Type-level assertions intentionally validate public core ID contracts.
import type {
  TestToneBypassParameterId,
  TestToneEnabledParameterId,
  TestToneFrequencyParameterId,
  TestToneLevelParameterId,
  TestToneParameterIds,
  TestToneProcessorId,
} from '@wavecraft/core';

type Equal<A, B> = ((<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2
  ? true
  : false) &
  ((<T>() => T extends B ? 1 : 2) extends <T>() => T extends A ? 1 : 2 ? true : false);

type Assert<T extends true> = T;
type NotNever<T> = [T] extends [never] ? false : true;

export type _TestToneProcessorIdIsNotNever = Assert<NotNever<TestToneProcessorId>>;
export type _TestToneProcessorIdIncludesTestTone = Assert<
  'test_tone' extends TestToneProcessorId ? true : false
>;

export type _TestToneBypassIdIsNotNever = Assert<NotNever<TestToneBypassParameterId>>;
export type _TestToneEnabledIdIsNotNever = Assert<NotNever<TestToneEnabledParameterId>>;
export type _TestToneFrequencyIdIsNotNever = Assert<NotNever<TestToneFrequencyParameterId>>;
export type _TestToneLevelIdIsNotNever = Assert<NotNever<TestToneLevelParameterId>>;

export type _TestToneBypassSuffixIsParameterId = Assert<
  'test_tone_bypass' extends TestToneBypassParameterId ? true : false
>;
export type _TestToneEnabledSuffixIsParameterId = Assert<
  'test_tone_enabled' extends TestToneEnabledParameterId ? true : false
>;
export type _TestToneFrequencySuffixIsParameterId = Assert<
  'test_tone_frequency' extends TestToneFrequencyParameterId ? true : false
>;
export type _TestToneLevelSuffixIsParameterId = Assert<
  'test_tone_level' extends TestToneLevelParameterId ? true : false
>;

export type _TestToneInvalidSuffixIsRejected = Assert<
  Equal<'test_tone_output_trim_db' extends TestToneFrequencyParameterId ? true : false, false>
>;

export type _TestToneParameterIdsContract = Assert<
  Equal<
    TestToneParameterIds,
    {
      readonly bypass: TestToneBypassParameterId;
      readonly enabled: TestToneEnabledParameterId;
      readonly frequency: TestToneFrequencyParameterId;
      readonly level: TestToneLevelParameterId;
    }
  >
>;
