import type { BypassProcessorId, LevelProcessorId, ParameterId } from '@wavecraft/core';

type Equal<A, B> = (
  (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false
) & (
  (<T>() => T extends B ? 1 : 2) extends (<T>() => T extends A ? 1 : 2) ? true : false
);

type Assert<T extends true> = T;
type NotNever<T> = [T] extends [never] ? false : true;

type GainLikeProcessorId = Extract<LevelProcessorId, BypassProcessorId>;

export type _GainLikeProcessorIdIsNotNever = Assert<NotNever<GainLikeProcessorId>>;
export type _GainLikeIncludesInputTrim = Assert<'input_trim' extends GainLikeProcessorId ? true : false>;
export type _GainLikeIncludesOutputGain = Assert<
  'output_gain' extends GainLikeProcessorId ? true : false
>;
export type _GainLikeExcludesExampleProcessor = Assert<
  Equal<'example_processor' extends GainLikeProcessorId ? true : false, false>
>;

export type _ValidGainLikeLevelSuffixIsParameterId = Assert<
  `${GainLikeProcessorId}_level` extends ParameterId ? true : false
>;
export type _InvalidGainLikeSuffixIsRejected = Assert<
  Equal<`${GainLikeProcessorId}_bla` extends ParameterId ? true : false, false>
>;
