//! 预导入模块

pub use crate::{
    codegen::{MigrationFile, MigrationType, SchemaCodegen, SchemaIrFunction, SchemaIrModule, SchemaIrParam},
    compiler::SchemaCompiler,
    error::{SchemaError, SchemaResult},
    ir::{
        AnnotationArgIr, AnnotationIr, EnumIr, EnumVariantIr, FieldIr, FieldTypeIr, MessageIr, ModelIr, RpcMethodType,
        SchemaConfigIr, SchemaIr, ServiceIr, ServiceMethodIr,
    },
    transformer::SchemaTransformer,
    validator::SchemaValidator,
};
