import { readFile, writeFile } from "node:fs/promises";
import rehypeRaw from "rehype-raw";
import rehypeStringify from "rehype-stringify";
import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import { unified } from "unified";
import {
	rehypeAddCri,
	rehypeFrontmatter,
	rehypeHeadingToSection,
	remarkHeadingAttr,
} from "./plugin";

const remarkPlugins = [remarkGfm, remarkHeadingAttr];
const rehypePlugins = [rehypeFrontmatter, rehypeHeadingToSection, rehypeAddCri];
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
