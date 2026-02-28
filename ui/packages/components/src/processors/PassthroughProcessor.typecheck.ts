import type {
  PassthroughBypassParameterId as CorePassthroughBypassParameterId,
  PassthroughParameterIds as CorePassthroughParameterIds,
  PassthroughProcessorId as CorePassthroughProcessorId,
} from '@wavecraft/core';
import type {
  PassthroughBypassParameterId,
  PassthroughProcessorId,
  PassthroughProcessorProps,
} from './PassthroughProcessor';

type Equal<A, B> = ((<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2
  ? true
  : false) &
  ((<T>() => T extends B ? 1 : 2) extends <T>() => T extends A ? 1 : 2 ? true : false);

type Assert<T extends true> = T;
type NotNever<T> = [T] extends [never] ? false : true;

export type _PassthroughProcessorIdIsNotNever = Assert<NotNever<PassthroughProcessorId>>;
export type _PassthroughBypassIdIsNotNever = Assert<NotNever<PassthroughBypassParameterId>>;
export type _ComponentProcessorIdMatchesCore = Assert<
  Equal<PassthroughProcessorId, CorePassthroughProcessorId>
>;
export type _ComponentBypassIdMatchesCore = Assert<
  Equal<PassthroughBypassParameterId, CorePassthroughBypassParameterId>
>;
export type _ComponentPropProcessorIdMatchesCore = Assert<
  Equal<PassthroughProcessorProps['processorId'], CorePassthroughProcessorId>
>;

export type _BypassSuffixIsAcceptedForPassthroughProcessorIds = Assert<
  `${PassthroughProcessorId}_bypass` extends CorePassthroughBypassParameterId ? true : false
>;

export type _PassthroughParameterIdsContract = Assert<
  Equal<
    CorePassthroughParameterIds,
    {
      readonly bypass: CorePassthroughBypassParameterId;
    }
  >
>;
