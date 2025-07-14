use sourcemap::SourceMap;
use thiserror::Error;
use swc_ecma_parser::{Parser, StringInput, Syntax};
use swc_common::{sync::Lrc, SourceMap as SwcSourceMap, FileName, Span};
use std::fs;
use std::collections::HashMap;
use swc_ecma_ast::Module;
use swc_common::Spanned;

#[derive(Error, Debug)]
pub enum SourcemapError {
    #[error("invalid JSON or sourcemap: {0}")]
    Parse(String),
}

/// Parse a sourcemap JSON string and return the `sourcemap::SourceMap` object.
pub fn parse_sourcemap(json: &str) -> Result<SourceMap, SourcemapError> {
    SourceMap::from_slice(json.as_bytes()).map_err(|e| SourcemapError::Parse(e.to_string()))
}

/// Convenience helper: return the list of original source paths contained in the map.
pub fn sources_list(sm: &SourceMap) -> Vec<String> {
    (0..sm.get_source_count())
        .filter_map(|i| sm.get_source(i).map(|s| s.to_string()))
        .collect()
}

/// Find a span in the module whose starting position matches the mapping line/col.
fn find_span_in_module(cm: &SwcSourceMap, module: &Module, line: usize, col: usize) -> Option<Span> {
    for item in &module.body {
        let span = item.span();
        let loc = cm.lookup_char_pos(span.lo());
        if loc.line == line + 1 && loc.col_display == col {
            return Some(span);
        }
    }
    None
}

/// Reconstruct original sources from a sourcemap and generated JS.
pub fn reconstruct_sources_with_swc(sm: &SourceMap, generated_js: &str) -> Result<(), SourcemapError> {
    // 1. Try to use sourcesContent if present
    let source_count = sm.get_source_count();
    let mut any_sources_content = false;
    for i in 0..source_count {
        if let Some(content) = sm.get_source_contents(i as u32) {
            let path = sm.get_source(i).unwrap_or("");
            if !path.is_empty() {
                fs::write(path, content).map_err(|e| SourcemapError::Parse(format!("Failed to write {}: {}", path, e)))?;
                println!("Wrote {} bytes to {} (sourcesContent)", content.len(), path);
                any_sources_content = true;
            }
        }
    }
    if any_sources_content {
        return Ok(());
    }

    // 2. If sourcesContent is missing, use SWC to parse the generated JS
    let cm: Lrc<SwcSourceMap> = Default::default();
    let js_owned = generated_js.to_owned();
    let fm = cm.new_source_file(FileName::Custom("generated.js".into()).into(), js_owned);
    let mut parser = Parser::new(Syntax::Es(Default::default()), StringInput::from(&*fm), None);
    let module = match parser.parse_module() {
        Ok(module) => module,
        Err(e) => {
            return Err(SourcemapError::Parse(format!("SWC parse error: {:?}", e)));
        }
    };
    println!("SWC parsed generated JS with {} top-level items", module.body.len());

    // Helper: extract code segment from span
    fn extract_segment(cm: &SwcSourceMap, fm: &swc_common::SourceFile, span: Span) -> Option<String> {
        let start = cm.lookup_byte_offset(span.lo()).pos.0 as usize;
        let end = cm.lookup_byte_offset(span.hi()).pos.0 as usize;
        if end > start && end <= fm.src.len() {
            Some(fm.src[start..end].to_string())
        } else {
            None
        }
    }

    // 3. Iterate over all mappings and collect segments for each original source
    let mut source_segments: HashMap<String, Vec<String>> = HashMap::new();
    let fm = cm.lookup_char_pos(module.span().lo()).file;
    for token in sm.tokens() {
        let src_idx = token.get_src_id();
        let src_path = sm.get_source(src_idx).unwrap_or("").to_string();
        let gen_line = token.get_dst_line() as usize;
        let gen_col = token.get_dst_col() as usize;
        // Attempt to find a span starting exactly at (gen_line, gen_col)
        let found = find_span_in_module(&cm, &module, gen_line, gen_col);
        let segment = if let Some(span) = found {
            extract_segment(&cm, &fm, span).unwrap_or_else(|| "[unextractable segment]".to_string())
        } else {
            println!("Warning: No AST node found for {}:{} in {}", gen_line, gen_col, src_path);
            format!("[segment for {}:{} in {}]", gen_line, gen_col, src_path)
        };
        source_segments.entry(src_path).or_default().push(segment);
    }

    // 4. Concatenate segments and write to file
    for (src_path, segments) in source_segments.iter() {
        let reconstructed = segments.join("");
        if !src_path.is_empty() {
            fs::write(src_path, &reconstructed)
                .map_err(|e| SourcemapError::Parse(format!("Failed to write {}: {}", src_path, e)))?;
            println!("Wrote {} bytes to {}", reconstructed.len(), src_path);
        }
    }
    Ok(())
} 