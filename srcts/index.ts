import { readFile, writeFile } from "node:fs/promises";
import { glob } from "fast-glob";
import rehypeFormat from "rehype-format";
import rehypeRaw from "rehype-raw";
import rehypeStringify from "rehype-stringify";
import remarkGfm from "remark-gfm";
import remarkHeadingAttrs from "remark-heading-attrs";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import remarkUnwrapImages from "remark-unwrap-images";
import { unified } from "unified";
import {
	rehypeAddCri,
	rehypeFrontmatter,
	rehypeWrapHeadingSection,
} from "./plugin";
import { createDir } from "./utils";

const remarkPlugins = [remarkGfm, remarkHeadingAttrs, remarkUnwrapImages];
const rehypePlugins = [
	rehypeFrontmatter,
	rehypeWrapHeadingSection,
	rehypeAddCri,
	rehypeFormat,
];
const processor = unified()
	.use(remarkParse)
	.use(remarkPlugins)
	.use(remarkRehype, {
		allowDangerousHtml: true,
	})
	.use(rehypeRaw)
	.use(rehypePlugins)
	.use(rehypeStringify);

const main = async () => {
	await createDir("../output-html");
	const entries = await glob("../output/**/*.md");
	entries.forEach(async (file) => {
		const result = await processor.process(await readFile(file, "utf-8"));
		const filename = file.replace(".md", ".html").split("/").pop();
		writeFile(`../output-html/${filename}`, result.toString());
	});
};

main();
