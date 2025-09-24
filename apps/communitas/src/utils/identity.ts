import { safeInvoke } from './tauri'

const FALLBACK_WORDS = [
  'ocean','forest','mountain','river','desert','valley','meadow','storm','cloud','wind',
  'moon','star','sun','comet','ember','shadow','flame','stone','metal','leaf',
  'wolf','eagle','lion','tiger','bear','hawk','otter','whale','dolphin','fox',
  'harbor','harvest','aurora','midnight','dawn','dusk','ember','grove','delta','summit'
]

const pickRandomWord = () => FALLBACK_WORDS[Math.floor(Math.random() * FALLBACK_WORDS.length)]

export const generateFourWordIdentity = async (): Promise<string> => {
  const generated = await safeInvoke<string>('generate_four_word_identity')
  if (generated && typeof generated === 'string') {
    return generated.trim()
  }

  return [pickRandomWord(), pickRandomWord(), pickRandomWord(), pickRandomWord()].join('-')
}

export const normalizeFourWords = (fourWords: string): [string, string, string, string] => {
  const parts = fourWords
    .split('-')
    .map((part) => part.trim().toLowerCase())
    .filter(Boolean)

  if (parts.length !== 4) {
    throw new Error('Invalid four-word identity format')
  }

  return [parts[0], parts[1], parts[2], parts[3]]
}

export const validateFourWords = async (fourWords: string): Promise<boolean> => {
  const response = await safeInvoke<boolean>('validate_four_word_identity', { four_words: fourWords })
  if (typeof response === 'boolean') return response
  return /^[a-z]+(-[a-z]+){3}$/.test(fourWords)
}
