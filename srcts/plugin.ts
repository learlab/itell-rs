import { Element, Root } from "hast";
import { h } from "hastscript";
import { SKIP, visit } from "unist-util-visit";
import yaml from "yaml";

export const rehypeWrapHeadingSection = () => {
	return (tree: Root) => {
		const sections: Element[] = [];
		let currentSection: Element | null = null;

		visit(tree, (node, index, parent) => {
			if (
				node.type === "element" &&
				node.tagName === "h2" &&
				index !== undefined
			) {
				if (currentSection) {
					sections.push(currentSection);
				}

				const id = node.properties.id;
				const className = node.properties.className;

				currentSection = h(
					"section",
					{
						class: "content-chunk",
						"data-chunk-slug": id,
						"aria-labelledby": id,
					},
					[
						{ type: "text", value: "\n\n" },
						h("h2", node.properties, [...node.children]),
						{ type: "text", value: "\n" },
					],
				);

				return [SKIP, index + 1];
			}

			if (currentSection && parent === tree && index !== undefined) {
				currentSection.children.push(node);
				return [SKIP, index + 1];
			}
		});

		if (currentSection) {
			sections.push(currentSection);
		}

		// Add newlines between sections
		const newTree = { type: "root", children: [] };
		sections.forEach((section, index) => {
			newTree.children.push(section);
			if (index < sections.length - 1) {
				newTree.children.push({ type: "text", value: "\n\n" });
			}
		});

		tree.children = newTree.children;
	};
};

export const rehypeFrontmatter = () => {
	return (tree: Root, file) => {
		const frontmatterRegex = /^---\s*\n([\s\S]*?)\n---\s*\n([\s\S]*)$/;
		const match = file.value.match(frontmatterRegex);

		if (match) {
			const [, frontmatterString] = match;
			const frontmatter = yaml.parse(frontmatterString);
			const cri = Object.fromEntries(
				frontmatter.cri.map((item) => [
					item.slug,
					{ question: item.question, answer: item.answer },
				]),
			);
			file.cri = cri;
		}
		visit(tree, "element", () => {
			return SKIP;
		});
	};
};

export const rehypeAddCri = () => {
	return (tree: Root, file) => {
		const cri = file.cri as Record<
			string,
			{ question: string; answer: string }
		>;
		visit(tree, "element", (node) => {
			if (node.tagName === "section") {
			}
			if (
				node.tagName === "section" &&
				node.properties &&
				node.properties.ariaLabelledBy
			) {
				const labelId = node.properties.ariaLabelledBy as string;

				if (cri[labelId]) {
					const { question, answer } = cri[labelId];

					const newElement: Element = {
						type: "element",
						tagName: "i-question",
						properties: {
							question: question,
							answer: answer,
						},
						children: [],
					};

					// Append the new element to the end of the section
					node.children.push(newElement, { type: "text", value: "\n\n" });
				}
			}
		});
	};
};
