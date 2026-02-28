import type {
  ToneFilterBypassParameterId,
  ToneFilterCutoffHzParameterId,
  ToneFilterModeParameterId,
  ToneFilterParameterIds,
  ToneFilterProcessorId,
  ToneFilterResonanceQParameterId,
} from '@wavecraft/core';

type Equal<A, B> = ((<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2
  ? true
  : false) &
  ((<T>() => T extends B ? 1 : 2) extends <T>() => T extends A ? 1 : 2 ? true : false);

type Assert<T extends true> = T;
type NotNever<T> = [T] extends [never] ? false : true;

export type _ToneFilterProcessorIdIsNotNever = Assert<NotNever<ToneFilterProcessorId>>;
export type _ToneFilterProcessorIdIncludesToneFilter = Assert<
  'tone_filter' extends ToneFilterProcessorId ? true : false
>;
export type _ToneFilterBypassIdIsNotNever = Assert<NotNever<ToneFilterBypassParameterId>>;
export type _ToneFilterModeIdIsNotNever = Assert<NotNever<ToneFilterModeParameterId>>;
export type _ToneFilterCutoffIdIsNotNever = Assert<NotNever<ToneFilterCutoffHzParameterId>>;
export type _ToneFilterResonanceIdIsNotNever = Assert<NotNever<ToneFilterResonanceQParameterId>>;

export type _ToneFilterBypassSuffixIsParameterId = Assert<
  'tone_filter_bypass' extends ToneFilterBypassParameterId ? true : false
>;
export type _ToneFilterModeSuffixIsParameterId = Assert<
  'tone_filter_mode' extends ToneFilterModeParameterId ? true : false
>;
export type _ToneFilterCutoffSuffixIsParameterId = Assert<
  'tone_filter_cutoff_hz' extends ToneFilterCutoffHzParameterId ? true : false
>;
export type _ToneFilterResonanceSuffixIsParameterId = Assert<
  'tone_filter_resonance_q' extends ToneFilterResonanceQParameterId ? true : false
>;

export type _ToneFilterInvalidSuffixIsRejected = Assert<
  Equal<'tone_filter_drive_db' extends ToneFilterBypassParameterId ? true : false, false>
>;

export type _ToneFilterParameterIdsContract = Assert<
  Equal<
    ToneFilterParameterIds,
    {
      readonly bypass: ToneFilterBypassParameterId;
      readonly mode: ToneFilterModeParameterId;
      readonly cutoffHz: ToneFilterCutoffHzParameterId;
      readonly resonanceQ: ToneFilterResonanceQParameterId;
    }
  >
>;
