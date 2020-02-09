use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use markedit::{Heading, Matcher, Rewriter, Writer};
use pulldown_cmark::Event;
use std::path::{Path, PathBuf};

fn known_markdown_files() -> impl Iterator<Item = PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let markedit_files = vec![
        manifest_dir.join("README.md"),
        manifest_dir.join("LICENSE_APACHE.md"),
    ];

    let blog_posts_pattern = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/benches/adventures.michaelfbryan.com/content/posts/**/*.md"
    );
    let blog_posts = glob::glob(blog_posts_pattern)
        .unwrap()
        .map(|entry| entry.unwrap());

    blog_posts.take(5).chain(markedit_files)
}

fn canonical_name(path: &Path) -> &str {
    let stem = path.file_stem().unwrap().to_str().unwrap();

    if stem == "index" {
        canonical_name(path.parent().unwrap())
    } else {
        stem
    }
}

pub fn rewriting(c: &mut Criterion) {
    let mut group = c.benchmark_group("Rewriting");

    for filename in known_markdown_files() {
        let src = std::fs::read_to_string(&filename).unwrap();
        let name = canonical_name(&filename);

        group
            .throughput(Throughput::Bytes(src.len() as u64))
            .bench_with_input(
                BenchmarkId::new("baseline parse", name),
                &src,
                |b, src| b.iter(|| markedit::parse(src).count()),
            )
            .bench_with_input(
                BenchmarkId::new("add text after each heading", name),
                &src,
                |b, src| {
                    b.iter(|| {
                        markedit::insert_markdown_before(
                            "## Sub-Heading",
                            Heading::with_level(2).falling_edge(),
                        )
                        .rewrite(markedit::parse(src))
                        .count()
                    })
                },
            )
            .bench_with_input(
                BenchmarkId::new("lowercase all text", name),
                &src,
                |b, src| {
                    b.iter(|| {
                        markedit::change_text(
                            |_| true,
                            |text| text.to_uppercase(),
                        )
                        .rewrite(markedit::parse(src))
                        .count()
                    })
                },
            )
            .bench_with_input(
                BenchmarkId::new("uppercase level 2 headings", name),
                &src,
                |b, src| {
                    b.iter(|| {
                        upper_case_header_text(2)
                            .rewrite(markedit::parse(src))
                            .count()
                    })
                },
            );
    }
}

fn upper_case_header_text<'src>(level: u32) -> impl Rewriter<'src> {
    let mut matcher = Heading::with_level(level);

    move |ev: Event<'src>, writer: &mut Writer<'src>| {
        if matcher.matches_event(&ev) {
            if let Event::Text(text) = ev {
                writer.push(Event::Text(
                    text.into_string().to_uppercase().into(),
                ));
                return;
            }
        }

        writer.push(ev);
    }
}

criterion_group!(benches, rewriting);
criterion_main!(benches);
