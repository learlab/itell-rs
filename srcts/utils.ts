import { access, mkdir, rmdir, unlink } from "node:fs/promises";

export const createDir = async (dir: string) => {
	try {
		await access(dir);
	} catch {
		await mkdir(dir, { recursive: true });
	}
};
