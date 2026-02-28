// eslint-disable-next-line no-restricted-imports -- Type-level assertions intentionally validate public core ID contracts.
import type {
  SoftClipBypassParameterId as CoreSoftClipBypassParameterId,
  SoftClipDriveDbParameterId as CoreSoftClipDriveDbParameterId,
  SoftClipMixParameterId as CoreSoftClipMixParameterId,
  SoftClipOutputDbParameterId as CoreSoftClipOutputDbParameterId,
  SoftClipParameterIds as CoreSoftClipParameterIds,
  SoftClipProcessorId as CoreSoftClipProcessorId,
  SoftClipToneParameterId as CoreSoftClipToneParameterId,
} from '@wavecraft/core';
import type { SaturatorProcessorProps } from './SaturatorProcessor';

type Equal<A, B> = ((<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2
  ? true
  : false) &
  ((<T>() => T extends B ? 1 : 2) extends <T>() => T extends A ? 1 : 2 ? true : false);

type Assert<T extends true> = T;
type NotNever<T> = [T] extends [never] ? false : true;

export type _SoftClipProcessorIdIsNotNever = Assert<NotNever<CoreSoftClipProcessorId>>;
export type _SoftClipBypassIdIsNotNever = Assert<NotNever<CoreSoftClipBypassParameterId>>;
export type _SoftClipDriveDbIdIsNotNever = Assert<NotNever<CoreSoftClipDriveDbParameterId>>;
export type _SoftClipOutputDbIdIsNotNever = Assert<NotNever<CoreSoftClipOutputDbParameterId>>;
export type _SoftClipMixIdIsNotNever = Assert<NotNever<CoreSoftClipMixParameterId>>;
export type _SoftClipToneIdIsNotNever = Assert<NotNever<CoreSoftClipToneParameterId>>;

export type _SaturatorProcessorIdIncludesSoftClip = Assert<
  'soft_clip' extends CoreSoftClipProcessorId ? true : false
>;

export type _SoftClipDriveSuffixIsAccepted = Assert<
  'soft_clip_drive_db' extends CoreSoftClipDriveDbParameterId ? true : false
>;

export type _SoftClipOutputSuffixIsAccepted = Assert<
  'soft_clip_output_db' extends CoreSoftClipOutputDbParameterId ? true : false
>;

export type _SoftClipMixSuffixIsAccepted = Assert<
  'soft_clip_mix' extends CoreSoftClipMixParameterId ? true : false
>;

export type _SoftClipToneSuffixIsAccepted = Assert<
  'soft_clip_tone' extends CoreSoftClipToneParameterId ? true : false
>;

export type _SaturatorPropsContract = Assert<
  Equal<
    SaturatorProcessorProps,
    {
      readonly hideWhenNotInSignalChain?: boolean;
      readonly className?: string;
    }
  >
>;

export type _SoftClipParameterIdsContract = Assert<
  Equal<
    CoreSoftClipParameterIds,
    {
      readonly bypass: CoreSoftClipBypassParameterId;
      readonly driveDb: CoreSoftClipDriveDbParameterId;
      readonly outputDb: CoreSoftClipOutputDbParameterId;
      readonly mix: CoreSoftClipMixParameterId;
      readonly tone: CoreSoftClipToneParameterId;
    }
  >
>;
