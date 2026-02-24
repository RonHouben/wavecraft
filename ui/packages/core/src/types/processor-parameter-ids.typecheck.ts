import type { BypassProcessorId, LevelProcessorId } from './processor-parameter-ids';
import type { ParameterId } from './parameters';
import type { ProcessorId } from './processors';
import type {
  BypassProcessorId as ExportedBypassProcessorId,
  LevelProcessorId as ExportedLevelProcessorId,
} from '../index';

type Equal<A, B> = ((<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2
  ? true
  : false) &
  ((<T>() => T extends B ? 1 : 2) extends <T>() => T extends A ? 1 : 2 ? true : false);

type Assert<T extends true> = T;
type NotNever<T> = [T] extends [never] ? false : true;

type GainLikeProcessorId = Extract<LevelProcessorId, BypassProcessorId>;

export type _LevelProcessorIdIsNotNever = Assert<NotNever<LevelProcessorId>>;
export type _BypassProcessorIdIsNotNever = Assert<NotNever<BypassProcessorId>>;
export type _GainLikeProcessorIdIsNotNever = Assert<NotNever<GainLikeProcessorId>>;
export type _ProcessorIdIsNotNever = Assert<NotNever<ProcessorId>>;
export type _ExportedLevelProcessorIdIsNotNever = Assert<NotNever<ExportedLevelProcessorId>>;
export type _ExportedBypassProcessorIdIsNotNever = Assert<NotNever<ExportedBypassProcessorId>>;

export type _ExportedLevelMatchesInternal = Assert<
  Equal<ExportedLevelProcessorId, LevelProcessorId>
>;

export type _ExportedBypassMatchesInternal = Assert<
  Equal<ExportedBypassProcessorId, BypassProcessorId>
>;

export type _GainLikeIncludesInputTrim = Assert<
  'input_trim' extends GainLikeProcessorId ? true : false
>;

export type _GainLikeIncludesOutputGain = Assert<
  'output_gain' extends GainLikeProcessorId ? true : false
>;

export type _InputTrimLevelSuffixIsAccepted = Assert<
  'input_trim_level' extends ParameterId ? true : false
>;

export type _OutputGainLevelSuffixIsAccepted = Assert<
  'output_gain_level' extends ParameterId ? true : false
>;

export type _InputTrimBypassSuffixIsAccepted = Assert<
  'input_trim_bypass' extends ParameterId ? true : false
>;

export type _OutputGainBypassSuffixIsAccepted = Assert<
  'output_gain_bypass' extends ParameterId ? true : false
>;

export type _ExampleProcessorLevelSuffixIsRejected = Assert<
  Equal<'example_processor_level' extends ParameterId ? true : false, false>
>;

export type _LevelProcessorExcludesExampleProcessor = Assert<
  Equal<'example_processor' extends LevelProcessorId ? true : false, false>
>;

export type _InvalidSuffixIsRejected = Assert<
  Equal<'input_trim_bla' extends ParameterId ? true : false, false>
>;
