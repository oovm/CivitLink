export interface DocNode {
    id: string;
    title: string;
    path: string;
    isDirectory: boolean;
    children?: DocNode[];
    parentId?: string;
    order?: number;
}

export interface DocMetadata {
    title?: string;
    order?: number;
}

import { docsModules as rawDocsModules } from "virtual:docs";

const docsModules: Record<string, string> = {};
for (const key in rawDocsModules) {
    const normalizedKey = key.replace(/\\/g, "/");
    docsModules[normalizedKey] = rawDocsModules[key];
}

console.log("Loaded docsModules:", Object.keys(docsModules));

function parseDocMetadata(content: string): DocMetadata {
    const metadata: DocMetadata = {};
    const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);

    if (frontmatterMatch) {
        const frontmatter = frontmatterMatch[1];
        const titleMatch = frontmatter.match(/^title:\s*(.+)$/m);
        const orderMatch = frontmatter.match(/^order:\s*(\d+)$/m);

        if (titleMatch) {
            metadata.title = titleMatch[1].trim().replace(/^['"]|['"]$/g, "");
        }
        if (orderMatch) {
            metadata.order = parseInt(orderMatch[1], 10);
        }
    }

    return metadata;
}

function normalizePath(path: string): string {
    return path.replace(/\\/g, "/");
}

function generateId(path: string): string {
    const normalizedPath = normalizePath(path);
    return normalizedPath.replace(/\.md$/, "").replace(/\//g, "-");
}

function getDocTitle(path: string, metadata: DocMetadata): string {
    if (metadata.title) {
        return metadata.title;
    }

    const normalizedPath = normalizePath(path);
    const filename = normalizedPath.split("/").pop()?.replace(".md", "") || "";

    const titleMap: Record<string, string> = {
        index: "首页",
        introduction: "介绍",
        "quick-start": "快速开始",
        features: "功能特性",
        grammar: "语法",
        guide: "指南",
        "rust-usage": "Rust 使用",
        maintenance: "维护",
        parser: "解析器",
        testing: "测试",
        faq: "常见问题",
        readme: "说明",
    };

    return titleMap[filename] || filename;
}

function buildDocTree(docs: DocNode[]): DocNode[] {
    const nodeMap: Record<string, DocNode> = {};
    const pathToNode: Record<string, DocNode> = {};
    const dirMap: Record<string, DocNode> = {};

    docs.forEach((doc) => {
        const normalizedPath = normalizePath(doc.path);
        const node = { ...doc, path: normalizedPath, children: [] };
        nodeMap[doc.id] = node;
        pathToNode[normalizedPath] = node;
    });

    const root: DocNode[] = [];

    docs.forEach((doc) => {
        const normalizedPath = normalizePath(doc.path);
        const pathParts = normalizedPath.split("/");

        if (pathParts.length === 1) {
            if (pathParts[0] === "index.md") {
                root.push(pathToNode[normalizedPath]);
            }
        } else {
            const dirPath = pathParts.slice(0, -1).join("/");
            if (!dirMap[dirPath]) {
                dirMap[dirPath] = {
                    id: dirPath.replace(/\//g, "-"),
                    title: pathParts[pathParts.length - 2],
                    path: dirPath,
                    isDirectory: true,
                    children: [],
                };

                if (pathParts.length === 2) {
                    root.push(dirMap[dirPath]);
                } else {
                    const parentDirPath = pathParts.slice(0, -2).join("/");
                    if (dirMap[parentDirPath]) {
                        dirMap[parentDirPath].children?.push(dirMap[dirPath]);
                    }
                }
            }

            dirMap[dirPath].children?.push(pathToNode[normalizedPath]);
        }
    });

    const sortNodes = (nodes: DocNode[]): DocNode[] => {
        return nodes
            .sort((a, b) => {
                if (a.isDirectory && !b.isDirectory) return -1;
                if (!a.isDirectory && b.isDirectory) return 1;
                if (a.path.endsWith("index.md") && !b.path.endsWith("index.md")) return -1;
                if (!a.path.endsWith("index.md") && b.path.endsWith("index.md")) return 1;
                return (a.order || 999) - (b.order || 999);
            })
            .map((node) => ({
                ...node,
                children: node.children ? sortNodes(node.children) : undefined,
            }));
    };

    return sortNodes(root);
}

export async function loadDocs(): Promise<DocNode[]> {
    console.log("docsModules keys:", Object.keys(docsModules));
    const docs: DocNode[] = [];

    for (const path in docsModules) {
        const content = docsModules[path] as string;
        const metadata = parseDocMetadata(content);
        const id = generateId(path);
        const title = getDocTitle(path, metadata);

        docs.push({
            id,
            title,
            path,
            isDirectory: false,
            order: metadata.order,
        });
    }

    console.log("Loaded docs:", docs);
    const tree = buildDocTree(docs);
    console.log("Built doc tree:", tree);
    return tree;
}

export async function getDocContent(path: string): Promise<string> {
    const normalizedPath = normalizePath(path);
    return (docsModules[normalizedPath] as string) || (docsModules[path] as string) || "";
}
