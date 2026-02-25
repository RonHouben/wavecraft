import type { ParameterId } from './parameters';

type ProcessorFromParamIdWithSuffix<
  TParamId extends string,
  Suffix extends string,
> = TParamId extends `${infer Proc}${Suffix}` ? Proc : never;

type ParameterIdsWithSuffix<Suffix extends string> = Extract<ParameterId, `${string}${Suffix}`>;

type ProcessorIdsWithParameterSuffix<Suffix extends string> = ProcessorFromParamIdWithSuffix<
  ParameterIdsWithSuffix<Suffix>,
  Suffix
>;

export type ProcessorIdForParameterSuffix<Suffix extends string> =
  ProcessorIdsWithParameterSuffix<Suffix>;

export type ParameterIdForProcessorSuffix<
  TProcessorId extends string,
  Suffix extends string,
> = Extract<ParameterId, `${TProcessorId}${Suffix}`>;

export type LevelProcessorId = ProcessorIdsWithParameterSuffix<'_level'>;

export type BypassProcessorId = ProcessorIdsWithParameterSuffix<'_bypass'>;

export type PassthroughProcessorId = Extract<BypassProcessorId, string>;

export type PassthroughBypassParameterId = ParameterIdForProcessorSuffix<
  PassthroughProcessorId,
  '_bypass'
>;

type ToneFilterProcessorIdDerived = ProcessorIdsWithParameterSuffix<'_bypass'> &
  ProcessorIdsWithParameterSuffix<'_mode'> &
  ProcessorIdsWithParameterSuffix<'_cutoff_hz'> &
  ProcessorIdsWithParameterSuffix<'_resonance_q'>;

export type ToneFilterProcessorId = Extract<ToneFilterProcessorIdDerived, string>;

export type ToneFilterBypassParameterId = ParameterIdForProcessorSuffix<
  ToneFilterProcessorId,
  '_bypass'
>;

export type ToneFilterModeParameterId = ParameterIdForProcessorSuffix<
  ToneFilterProcessorId,
  '_mode'
>;

export type ToneFilterCutoffHzParameterId = ParameterIdForProcessorSuffix<
  ToneFilterProcessorId,
  '_cutoff_hz'
>;

export type ToneFilterResonanceQParameterId = ParameterIdForProcessorSuffix<
  ToneFilterProcessorId,
  '_resonance_q'
>;

type SoftClipProcessorIdDerived = ProcessorIdsWithParameterSuffix<'_bypass'> &
  ProcessorIdsWithParameterSuffix<'_drive_db'> &
  ProcessorIdsWithParameterSuffix<'_output_trim_db'>;

export type SoftClipProcessorId = Extract<SoftClipProcessorIdDerived, string>;

export type SoftClipBypassParameterId = ParameterIdForProcessorSuffix<
  SoftClipProcessorId,
  '_bypass'
>;

export type SoftClipDriveDbParameterId = ParameterIdForProcessorSuffix<
  SoftClipProcessorId,
  '_drive_db'
>;

export type SoftClipOutputTrimDbParameterId = ParameterIdForProcessorSuffix<
  SoftClipProcessorId,
  '_output_trim_db'
>;

type TestToneProcessorIdDerived = ProcessorIdsWithParameterSuffix<'_bypass'> &
  ProcessorIdsWithParameterSuffix<'_enabled'> &
  ProcessorIdsWithParameterSuffix<'_frequency'> &
  ProcessorIdsWithParameterSuffix<'_level'>;

export type TestToneProcessorId = Extract<TestToneProcessorIdDerived, string>;

export type TestToneBypassParameterId = ParameterIdForProcessorSuffix<
  TestToneProcessorId,
  '_bypass'
>;

export type TestToneFrequencyParameterId = ParameterIdForProcessorSuffix<
  TestToneProcessorId,
  '_frequency'
>;

export type TestToneEnabledParameterId = ParameterIdForProcessorSuffix<
  TestToneProcessorId,
  '_enabled'
>;

export type TestToneLevelParameterId = ParameterIdForProcessorSuffix<TestToneProcessorId, '_level'>;

export interface ToneFilterParameterIds {
  readonly bypass: ToneFilterBypassParameterId;
  readonly mode: ToneFilterModeParameterId;
  readonly cutoffHz: ToneFilterCutoffHzParameterId;
  readonly resonanceQ: ToneFilterResonanceQParameterId;
}

export interface SoftClipParameterIds {
  readonly bypass: SoftClipBypassParameterId;
  readonly driveDb: SoftClipDriveDbParameterId;
  readonly outputTrimDb: SoftClipOutputTrimDbParameterId;
}

export interface PassthroughParameterIds {
  readonly bypass: PassthroughBypassParameterId;
}

export interface TestToneParameterIds {
  readonly bypass: TestToneBypassParameterId;
  readonly enabled: TestToneEnabledParameterId;
  readonly frequency: TestToneFrequencyParameterId;
  readonly level: TestToneLevelParameterId;
}
