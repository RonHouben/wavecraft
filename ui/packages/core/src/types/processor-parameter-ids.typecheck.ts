import type {
  BypassProcessorId,
  LevelProcessorId,
  PassthroughBypassParameterId,
  PassthroughParameterIds,
  PassthroughProcessorId,
  SoftClipBypassParameterId,
  SoftClipDriveDbParameterId,
  SoftClipOutputTrimDbParameterId,
  SoftClipParameterIds,
  SoftClipProcessorId,
  TestToneBypassParameterId,
  TestToneEnabledParameterId,
  TestToneFrequencyParameterId,
  TestToneLevelParameterId,
  TestToneParameterIds,
  TestToneProcessorId,
  ToneFilterBypassParameterId,
  ToneFilterCutoffHzParameterId,
  ToneFilterModeParameterId,
  ToneFilterProcessorId,
  ToneFilterResonanceQParameterId,
} from './processor-parameter-ids';
import type { ParameterId } from './parameters';
import type { ProcessorId } from './processors';
import type {
  BypassProcessorId as ExportedBypassProcessorId,
  LevelProcessorId as ExportedLevelProcessorId,
  PassthroughBypassParameterId as ExportedPassthroughBypassParameterId,
  PassthroughParameterIds as ExportedPassthroughParameterIds,
  PassthroughProcessorId as ExportedPassthroughProcessorId,
  SoftClipBypassParameterId as ExportedSoftClipBypassParameterId,
  SoftClipDriveDbParameterId as ExportedSoftClipDriveDbParameterId,
  SoftClipOutputTrimDbParameterId as ExportedSoftClipOutputTrimDbParameterId,
  SoftClipParameterIds as ExportedSoftClipParameterIds,
  SoftClipProcessorId as ExportedSoftClipProcessorId,
  TestToneBypassParameterId as ExportedTestToneBypassParameterId,
  TestToneEnabledParameterId as ExportedTestToneEnabledParameterId,
  TestToneFrequencyParameterId as ExportedTestToneFrequencyParameterId,
  TestToneLevelParameterId as ExportedTestToneLevelParameterId,
  TestToneParameterIds as ExportedTestToneParameterIds,
  TestToneProcessorId as ExportedTestToneProcessorId,
  ToneFilterBypassParameterId as ExportedToneFilterBypassParameterId,
  ToneFilterCutoffHzParameterId as ExportedToneFilterCutoffHzParameterId,
  ToneFilterModeParameterId as ExportedToneFilterModeParameterId,
  ToneFilterProcessorId as ExportedToneFilterProcessorId,
  ToneFilterResonanceQParameterId as ExportedToneFilterResonanceQParameterId,
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
export type _PassthroughProcessorIdIsNotNever = Assert<NotNever<PassthroughProcessorId>>;
export type _PassthroughBypassParameterIdIsNotNever = Assert<
  NotNever<PassthroughBypassParameterId>
>;
export type _ExportedPassthroughProcessorIdIsNotNever = Assert<
  NotNever<ExportedPassthroughProcessorId>
>;
export type _ExportedPassthroughBypassParameterIdIsNotNever = Assert<
  NotNever<ExportedPassthroughBypassParameterId>
>;
export type _ToneFilterProcessorIdIsNotNever = Assert<NotNever<ToneFilterProcessorId>>;
export type _ToneFilterBypassParameterIdIsNotNever = Assert<NotNever<ToneFilterBypassParameterId>>;
export type _ToneFilterModeParameterIdIsNotNever = Assert<NotNever<ToneFilterModeParameterId>>;
export type _ToneFilterCutoffParameterIdIsNotNever = Assert<
  NotNever<ToneFilterCutoffHzParameterId>
>;
export type _ToneFilterResonanceParameterIdIsNotNever = Assert<
  NotNever<ToneFilterResonanceQParameterId>
>;
export type _ExportedToneFilterProcessorIdIsNotNever = Assert<NotNever<ExportedToneFilterProcessorId>>;
export type _ExportedToneFilterBypassParameterIdIsNotNever = Assert<
  NotNever<ExportedToneFilterBypassParameterId>
>;
export type _ExportedToneFilterModeParameterIdIsNotNever = Assert<
  NotNever<ExportedToneFilterModeParameterId>
>;
export type _ExportedToneFilterCutoffParameterIdIsNotNever = Assert<
  NotNever<ExportedToneFilterCutoffHzParameterId>
>;
export type _ExportedToneFilterResonanceParameterIdIsNotNever = Assert<
  NotNever<ExportedToneFilterResonanceQParameterId>
>;
export type _SoftClipProcessorIdIsNotNever = Assert<NotNever<SoftClipProcessorId>>;
export type _SoftClipBypassParameterIdIsNotNever = Assert<NotNever<SoftClipBypassParameterId>>;
export type _SoftClipDriveDbParameterIdIsNotNever = Assert<NotNever<SoftClipDriveDbParameterId>>;
export type _SoftClipOutputTrimDbParameterIdIsNotNever = Assert<
  NotNever<SoftClipOutputTrimDbParameterId>
>;
export type _ExportedSoftClipProcessorIdIsNotNever = Assert<NotNever<ExportedSoftClipProcessorId>>;
export type _ExportedSoftClipBypassParameterIdIsNotNever = Assert<
  NotNever<ExportedSoftClipBypassParameterId>
>;
export type _ExportedSoftClipDriveDbParameterIdIsNotNever = Assert<
  NotNever<ExportedSoftClipDriveDbParameterId>
>;
export type _ExportedSoftClipOutputTrimDbParameterIdIsNotNever = Assert<
  NotNever<ExportedSoftClipOutputTrimDbParameterId>
>;
export type _TestToneProcessorIdIsNotNever = Assert<NotNever<TestToneProcessorId>>;
export type _TestToneBypassParameterIdIsNotNever = Assert<NotNever<TestToneBypassParameterId>>;
export type _TestToneEnabledParameterIdIsNotNever = Assert<NotNever<TestToneEnabledParameterId>>;
export type _TestToneFrequencyParameterIdIsNotNever = Assert<NotNever<TestToneFrequencyParameterId>>;
export type _TestToneLevelParameterIdIsNotNever = Assert<NotNever<TestToneLevelParameterId>>;
export type _ExportedTestToneProcessorIdIsNotNever = Assert<NotNever<ExportedTestToneProcessorId>>;
export type _ExportedTestToneBypassParameterIdIsNotNever = Assert<
  NotNever<ExportedTestToneBypassParameterId>
>;
export type _ExportedTestToneEnabledParameterIdIsNotNever = Assert<
  NotNever<ExportedTestToneEnabledParameterId>
>;
export type _ExportedTestToneFrequencyParameterIdIsNotNever = Assert<
  NotNever<ExportedTestToneFrequencyParameterId>
>;
export type _ExportedTestToneLevelParameterIdIsNotNever = Assert<
  NotNever<ExportedTestToneLevelParameterId>
>;

export type _ExportedLevelMatchesInternal = Assert<
  Equal<ExportedLevelProcessorId, LevelProcessorId>
>;

export type _ExportedBypassMatchesInternal = Assert<
  Equal<ExportedBypassProcessorId, BypassProcessorId>
>;

export type _ExportedPassthroughProcessorMatchesInternal = Assert<
  Equal<ExportedPassthroughProcessorId, PassthroughProcessorId>
>;

export type _ExportedPassthroughBypassMatchesInternal = Assert<
  Equal<ExportedPassthroughBypassParameterId, PassthroughBypassParameterId>
>;

export type _ExportedPassthroughParameterIdsMatchInternal = Assert<
  Equal<ExportedPassthroughParameterIds, PassthroughParameterIds>
>;

export type _ExportedToneFilterProcessorMatchesInternal = Assert<
  Equal<ExportedToneFilterProcessorId, ToneFilterProcessorId>
>;

export type _ExportedToneFilterBypassMatchesInternal = Assert<
  Equal<ExportedToneFilterBypassParameterId, ToneFilterBypassParameterId>
>;

export type _ExportedToneFilterModeMatchesInternal = Assert<
  Equal<ExportedToneFilterModeParameterId, ToneFilterModeParameterId>
>;

export type _ExportedToneFilterCutoffMatchesInternal = Assert<
  Equal<ExportedToneFilterCutoffHzParameterId, ToneFilterCutoffHzParameterId>
>;

export type _ExportedToneFilterResonanceMatchesInternal = Assert<
  Equal<ExportedToneFilterResonanceQParameterId, ToneFilterResonanceQParameterId>
>;

export type _ExportedSoftClipProcessorMatchesInternal = Assert<
  Equal<ExportedSoftClipProcessorId, SoftClipProcessorId>
>;

export type _ExportedSoftClipBypassMatchesInternal = Assert<
  Equal<ExportedSoftClipBypassParameterId, SoftClipBypassParameterId>
>;

export type _ExportedSoftClipDriveDbMatchesInternal = Assert<
  Equal<ExportedSoftClipDriveDbParameterId, SoftClipDriveDbParameterId>
>;

export type _ExportedSoftClipOutputTrimDbMatchesInternal = Assert<
  Equal<ExportedSoftClipOutputTrimDbParameterId, SoftClipOutputTrimDbParameterId>
>;

export type _ExportedSoftClipParameterIdsMatchInternal = Assert<
  Equal<ExportedSoftClipParameterIds, SoftClipParameterIds>
>;

export type _ExportedTestToneProcessorMatchesInternal = Assert<
  Equal<ExportedTestToneProcessorId, TestToneProcessorId>
>;

export type _ExportedTestToneBypassMatchesInternal = Assert<
  Equal<ExportedTestToneBypassParameterId, TestToneBypassParameterId>
>;

export type _ExportedTestToneEnabledMatchesInternal = Assert<
  Equal<ExportedTestToneEnabledParameterId, TestToneEnabledParameterId>
>;

export type _ExportedTestToneFrequencyMatchesInternal = Assert<
  Equal<ExportedTestToneFrequencyParameterId, TestToneFrequencyParameterId>
>;

export type _ExportedTestToneLevelMatchesInternal = Assert<
  Equal<ExportedTestToneLevelParameterId, TestToneLevelParameterId>
>;

export type _ExportedTestToneParameterIdsMatchInternal = Assert<
  Equal<ExportedTestToneParameterIds, TestToneParameterIds>
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

export type _PassthroughBypassSuffixPatternIsAccepted = Assert<
  `${BypassProcessorId}_bypass` extends PassthroughBypassParameterId ? true : false
>;

export type _ToneFilterProcessorIdIncludesToneFilter = Assert<
  'tone_filter' extends ToneFilterProcessorId ? true : false
>;

export type _ToneFilterBypassSuffixIsAccepted = Assert<
  'tone_filter_bypass' extends ToneFilterBypassParameterId ? true : false
>;

export type _ToneFilterModeSuffixIsAccepted = Assert<
  'tone_filter_mode' extends ToneFilterModeParameterId ? true : false
>;

export type _ToneFilterCutoffSuffixIsAccepted = Assert<
  'tone_filter_cutoff_hz' extends ToneFilterCutoffHzParameterId ? true : false
>;

export type _ToneFilterResonanceSuffixIsAccepted = Assert<
  'tone_filter_resonance_q' extends ToneFilterResonanceQParameterId ? true : false
>;

export type _ToneFilterInvalidSuffixIsRejected = Assert<
  Equal<'tone_filter_drive_db' extends ToneFilterBypassParameterId ? true : false, false>
>;

export type _SoftClipProcessorIdIncludesSoftClip = Assert<
  'soft_clip' extends SoftClipProcessorId ? true : false
>;

export type _SoftClipBypassSuffixIsAccepted = Assert<
  'soft_clip_bypass' extends SoftClipBypassParameterId ? true : false
>;

export type _SoftClipDriveDbSuffixIsAccepted = Assert<
  'soft_clip_drive_db' extends SoftClipDriveDbParameterId ? true : false
>;

export type _SoftClipOutputTrimDbSuffixIsAccepted = Assert<
  'soft_clip_output_trim_db' extends SoftClipOutputTrimDbParameterId ? true : false
>;

export type _SoftClipInvalidSuffixIsRejected = Assert<
  Equal<'soft_clip_level' extends SoftClipDriveDbParameterId ? true : false, false>
>;

export type _TestToneProcessorIdIncludesTestTone = Assert<
  'test_tone' extends TestToneProcessorId ? true : false
>;

export type _TestToneBypassSuffixIsAccepted = Assert<
  'test_tone_bypass' extends TestToneBypassParameterId ? true : false
>;

export type _TestToneFrequencySuffixIsAccepted = Assert<
  'test_tone_frequency' extends TestToneFrequencyParameterId ? true : false
>;

export type _TestToneEnabledSuffixIsAccepted = Assert<
  'test_tone_enabled' extends TestToneEnabledParameterId ? true : false
>;

export type _TestToneLevelSuffixIsAccepted = Assert<
  'test_tone_level' extends TestToneLevelParameterId ? true : false
>;

export type _TestToneInvalidSuffixIsRejected = Assert<
  Equal<'test_tone_drive_db' extends TestToneFrequencyParameterId ? true : false, false>
>;

export type _SoftClipParameterIdsContract = Assert<
  Equal<
    SoftClipParameterIds,
    {
      readonly bypass: SoftClipBypassParameterId;
      readonly driveDb: SoftClipDriveDbParameterId;
      readonly outputTrimDb: SoftClipOutputTrimDbParameterId;
    }
  >
>;

export type _PassthroughParameterIdsContract = Assert<
  Equal<
    PassthroughParameterIds,
    {
      readonly bypass: PassthroughBypassParameterId;
    }
  >
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

export type _ExampleProcessorLevelSuffixIsRejected = Assert<
  Equal<'example_processor_level' extends ParameterId ? true : false, false>
>;

export type _LevelProcessorExcludesExampleProcessor = Assert<
  Equal<'example_processor' extends LevelProcessorId ? true : false, false>
>;

export type _InvalidSuffixIsRejected = Assert<
  Equal<'input_trim_bla' extends ParameterId ? true : false, false>
>;
