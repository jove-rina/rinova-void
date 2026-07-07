/**
 * image-file-load.ts
 * 从本地路径或 File 加载图片元数据
 */
import { readImageFile } from '@/api/image-editor'

const IMAGE_EXT = /\.(png|jpe?g|gif|webp|bmp|tiff?|ico|avif|svg)$/i

export const isImagePath = (path: string): boolean => IMAGE_EXT.test(path)

export const isImageFile = (file: File): boolean =>
  file.type.startsWith('image/') || IMAGE_EXT.test(file.name)

export const fileFromPath = async (path: string): Promise<File> => {
  const result = await readImageFile(path)
  const bytes = new Uint8Array(result.bytes)
  const type = guessMimeFromName(result.name)
  return new File([bytes], result.name, { type })
}

const guessMimeFromName = (name: string): string => {
  const lower = name.toLowerCase()
  if (lower.endsWith('.png')) return 'image/png'
  if (lower.endsWith('.jpg') || lower.endsWith('.jpeg')) return 'image/jpeg'
  if (lower.endsWith('.webp')) return 'image/webp'
  if (lower.endsWith('.gif')) return 'image/gif'
  if (lower.endsWith('.bmp')) return 'image/bmp'
  if (lower.endsWith('.svg')) return 'image/svg+xml'
  return 'application/octet-stream'
}

export const readImageMeta = async (
  file: File,
): Promise<{ file: File; name: string; width: number; height: number }> => {
  const bitmap = await createImageBitmap(file)
  const meta = {
    file,
    name: file.name,
    width: bitmap.width,
    height: bitmap.height,
  }
  bitmap.close()
  return meta
}

export const bytesToImageFile = (bytes: number[], name: string): File => {
  const data = new Uint8Array(bytes)
  const type = guessMimeFromName(name)
  const file = new File([data], name, { type })
  return file
}
