import { readBinaryFile } from '@tauri-apps/api/fs';
// Read the image file in the `$RESOURCEDIR/avatar.png` path

// Each file type has their own byte header, indicating the file of the file.
// https://en.wikipedia.org/wiki/List_of_file_signatures
export const fileHeaderValidationOptions: any = {
	png: [
		{
			offset: 0,
			length: 8,
			valid: [[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]],
		},
	],
	jpeg: [
		{
			offset: 0,
			length: 3,
			valid: [[0xff, 0xd8, 0xff]],
		},
	],
	jpg: [
		{
			offset: 0,
			length: 3,
			valid: [[0xff, 0xd8, 0xff]],
		},
	],
	mp3: [
		{
			offset: 0,
			length: 2,
			valid: [
				[0xff, 0xfb],
				[0xff, 0xf3],
				[0xff, 0xf2],
			],
		},
		{
			offset: 0,
			length: 3,
			valid: [[0x49, 0x44, 0x33]],
		},
	],
	wav: [
		{
			offset: 0,
			length: 12,
			valid: [
				[
					0x52,
					0x49,
					0x46,
					0x46,
					null,
					null,
					null,
					null,
					0x57,
					0x41,
					0x56,
					0x45,
				],
			],
		},
	],
	mp4: [
		{
			offset: 4,
			length: 8,
			valid: [[0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6f, 0x6d]],
		},
	],
	doc: [
		{
			offset: 0,
			length: 8,
			valid: [[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]],
		},
		{
			offset: 0,
			length: 4,
			valid: [[0x0d, 0x44, 0x4f, 0x43]],
		},
	],
	docx: [
		{
			offset: 0,
			length: 4,
			valid: [
				[0x50, 0x4b, 0x03, 0x04],
				[0x50, 0x4b, 0x03, 0x06],
				[0x50, 0x4b, 0x03, 0x08],
			],
		},
	],
	odt: [
		{
			offset: 0,
			length: 4,
			valid: [
				[0x50, 0x4b, 0x03, 0x04],
				[0x50, 0x4b, 0x03, 0x06],
				[0x50, 0x4b, 0x03, 0x08],
			],
		},
	],
	odp: [
		{
			offset: 0,
			length: 4,
			valid: [
				[0x50, 0x4b, 0x03, 0x04],
				[0x50, 0x4b, 0x03, 0x06],
				[0x50, 0x4b, 0x03, 0x08],
			],
		},
	],
	pdf: [
		{
			offset: 0,
			length: 5,
			valid: [[0x25, 0x50, 0x44, 0x46, 0x2d]],
		},
	],
	txt: [
		{
			offset: 0,
			length: 3,
			valid: [
				[0xef, 0xbb, 0xbf],
				[0x0e, 0xfe, 0xff],
			],
		},
		{
			offset: 0,
			length: 2,
			valid: [
				[0xff, 0xfe],
				[0xfe, 0xff],
			],
		},
		{
			offset: 0,
			length: 4,
			valid: [
				[0x00, 0x00, 0xfe, 0xff],
				[0xff, 0xfe, 0x00, 0x00],
			],
		},
	],
	pcap: [
		{
			offset: 0,
			length: 4,
			valid: [
				[0xd4, 0xc3, 0xb2, 0xa1],
				[0xa1, 0xb2, 0xc3, 0xd4],
				[0x4d, 0x3c, 0xb2, 0xa1],
				[0xa1, 0xb2, 0x3c, 0x4d],
			],
		},
	],
};

export const pcapAllowedExtensions = ['pcap'];

// This fuction takes a byte signature of the file, in order to validate if the file is really a type of file declared by file extension.
export const validateFileHeader = (byteHeader: Uint8Array, type: string) => {
	const possibleValidations = fileHeaderValidationOptions[type];
	if (possibleValidations === null) return true;
	if (possibleValidations === undefined) return false;
	for (const { offset, length, valid } of possibleValidations) {
		const headerWindow = byteHeader.slice(offset, offset + length);
		for (const validPattern of valid) {
			for (
				let byteIndex = 0;
				byteIndex < headerWindow.byteLength;
				byteIndex++
			) {
				if (validPattern[byteIndex] === null) continue;
				if (headerWindow[byteIndex] !== validPattern[byteIndex])
					return false;
			}
		}
	}
	return true;
};
export const isValidFile = async (file: string) => {
	const fileType = 'pcap';
	const bytes = readBinaryFile(file);
	const isValid = validateFileHeader(await bytes, fileType);
	if (isValid) {
		return true;
	}
	return false;
};
