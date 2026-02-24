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

export type LevelProcessorId = ProcessorIdsWithParameterSuffix<'_level'>;

export type BypassProcessorId = ProcessorIdsWithParameterSuffix<'_bypass'>;
