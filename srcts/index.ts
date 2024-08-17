import { readFile, writeFile } from "node:fs/promises";
import rehypeRaw from "rehype-raw";
import rehypeStringify from "rehype-stringify";
import remarkFrontmatter from "remark-frontmatter";
import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import { unified } from "unified";
import { rehypeHeadingToSection, remarkHeadingAttr } from "./plugin";

const remarkPlugins = [remarkFrontmatter, remarkGfm, remarkHeadingAttr];
const rehypePlugins = [rehypeHeadingToSection];
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
	const result = await processor.process(
		await readFile("../output/2-program-structure.md", "utf-8"),
	);
	writeFile("test.html", result.toString());
};

main();
