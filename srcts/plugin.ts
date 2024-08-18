import { Element, Root } from "hast";
import { h } from "hastscript";
import { SKIP, visit } from "unist-util-visit";
import yaml from "yaml";

function rehypeWrapHeadingSection() {
	return (tree: Root, file) => {
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
						{ type: "text", value: "\n" },
						h("h2", { id, class: className }, [...node.children]),
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
				newTree.children.push({ type: "text", value: "\n" });
			}
		});

		tree.children = newTree.children;
	};
}

export function rehypeFrontmatter() {
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
		visit(tree, "element", (node, index, parent) => {
			return SKIP;
		});
	};
}

export function rehypeAddCri() {
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
					node.children.push(newElement, { type: "text", value: "\n" });
				}
			}
		});
	};
}

const attributeRegex = / {(?<attributes>[^}]+)}$/;
const idRegex = /#(?<id>[^\s}]+)/;
const classRegex = /\.(?<className>[^\s}]+)/g;
const keyValueRegex = /(?<key>[^\s=]+)=(?<value>[^\s}]+)/g;

const remarkHeadingAttr = () => {
	return (node: any) => {
		visit(node, "heading", (node: any) => {
			const textNode = node.children.at(-1);
			if (textNode?.type !== "text") {
				return SKIP;
			}

			const text = textNode.value.trimEnd();
			const matched = attributeRegex.exec(text);
			if (!matched) {
				return SKIP;
			}

			const { attributes } = matched.groups!;
			textNode.value = text.slice(0, matched.index);

			const hProperties: Record<string, any> = {};
			const classes: string[] = [];

			// Extract id
			const idMatch = idRegex.exec(attributes);
			if (idMatch) {
				const { id } = idMatch.groups!;
				hProperties.id = id;
			}

			// Extract classes
			let classMatch;
			while ((classMatch = classRegex.exec(attributes)) !== null) {
				const { className } = classMatch.groups!;
				classes.push(className);
			}
			if (classes.length > 0) {
				hProperties.class = classes.join(" ");
			}

			// Extract key-value pairs
			let keyValueMatch;
			while ((keyValueMatch = keyValueRegex.exec(attributes)) !== null) {
				const { key, value } = keyValueMatch.groups!;
				const camelCaseKey = `data${key.charAt(0).toUpperCase() + key.slice(1)}`;
				hProperties[camelCaseKey] = value;
			}

			node.data ??= {};
			node.data.hProperties = hProperties;
		});
	};
};

export {
	rehypeWrapHeadingSection as rehypeHeadingToSection,
	remarkHeadingAttr,
};
