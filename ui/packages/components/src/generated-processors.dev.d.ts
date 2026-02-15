/**
 * Development-only type loader for generated processor ID augmentation.
 *
 * This ensures TS projects rooted at `ui/packages/components` see
 * `ProcessorId` as a literal union from `sdk-template/ui/src/generated/processors.ts`
 * during monorepo development.
 */
import '../../../../sdk-template/ui/src/generated/processors';
