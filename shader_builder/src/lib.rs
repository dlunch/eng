use std::fs;
use std::io;
use std::path::Path;
use std::str;

pub fn build_shaders(shaders_base: &Path) -> io::Result<()> {
    for entry in shaders_base.read_dir()? {
        let path = entry?.path();
        let ext = path.extension().unwrap().to_str().unwrap();
        if ext == "frag" || ext == "vert" {
            let compiled_filename = format!("{}.spv", path.to_str().unwrap());
            let compilation = compile_shader(&path, ext)?;
            save_shader(Path::new(&compiled_filename), compilation)?;

            println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
            println!("cargo:rerun-if-changed={}", compiled_filename);
        }
    }

    Ok(())
}

fn compile_shader(path: &Path, ext: &str) -> io::Result<shaderc::CompilationArtifact> {
    let code = fs::read(path)?;

    let stage = match ext {
        "vert" => shaderc::ShaderKind::Vertex,
        "frag" => shaderc::ShaderKind::Fragment,
        _ => panic!(),
    };

    let mut compile_options = shaderc::CompileOptions::new().unwrap();
    compile_options.set_include_callback(|name, _, _, _| {
        let path = path.parent().unwrap().to_owned().join(name);
        let code = fs::read(path).unwrap();

        Ok(shaderc::ResolvedInclude {
            resolved_name: name.to_owned(),
            content: str::from_utf8(&code).unwrap().to_owned(),
        })
    });

    let file_name = path.file_name().unwrap().to_str().unwrap();
    let mut compiler = shaderc::Compiler::new().unwrap();
    Ok(compiler
        .compile_into_spirv(str::from_utf8(&code).unwrap(), stage, file_name, "main", Some(&compile_options))
        .unwrap())
}

fn save_shader(path: &Path, compilation: shaderc::CompilationArtifact) -> io::Result<()> {
    let code = compilation.as_binary_u8();

    fs::write(path, &code)
}
