import Editor, { loader } from '@monaco-editor/react'
import * as monaco from 'monaco-editor/editor/editor.api.js'
import EditorWorker from 'monaco-editor/editor/editor.worker.js?worker'

self.MonacoEnvironment = { getWorker: () => new EditorWorker() }
loader.config({ monaco })
monaco.languages.register({ id: 'jocky' })
monaco.languages.setMonarchTokensProvider('jocky', { tokenizer: { root: [
  [/\b(investigation|target|host|agent|collect|as|analyze|where|contains|equals|report|include|true|false)\b/, 'keyword'],
  [/"([^"\\]|\\.)*"/, 'string'], [/\/\/.*$/, 'comment'], [/\b\d+\b/, 'number'],
] } })

export default Editor
