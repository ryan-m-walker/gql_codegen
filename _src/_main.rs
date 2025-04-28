use std::{collections::HashMap, fs};

use generator::{
    ts_operation_types::TsOperationsTypeGenerator, ts_schema_types::TsSchemaTypesGenerator,
    zod_schemas::ZodSchemasGenerator, CodeGenerator,
};
use processor::Processor;

mod generator;
mod processor;

fn main() {
    // let schema_src = fs::read_to_string("schema.graphql").expect("Unable to read file");
    // let document_src = fs::read_to_string("document.graphql").expect("Unable to read file");

    // let generators = HashMap::new();
    // generators.insert("ts_schema_types", Box::new(TsSchemaTypesGenerator));
    // generators.insert("ts_operations_type", Box::new(TsOperationsTypeGenerator));
    // generators.insert("zod_schemas", Box::new(ZodSchemasGenerator));

    let ts_schema_types_generator = TsSchemaTypesGenerator;
    let ts_operations_type_generator = TsOperationsTypeGenerator;
    let zod_schema_generator = ZodSchemasGenerator;

    let result = Processor::new(&schema_src)
        .add_document("query.graphql", &document_src)
        .add_generator(Box::new(ts_schema_types_generator))
        .add_generator(Box::new(ts_operations_type_generator))
        .add_generator(Box::new(zod_schema_generator))
        .process();

    if let Some(result) = result {
        println!("\n{}", &result);
        let write_result = fs::write("output.ts", result);

        if let Err(error) = write_result {
            println!("An error occurred while trying to write codegen file");
            println!("{:?}", error);
        }
    }
}
