import languages from './languages.json';

/**
 * Interface defining language patterns and detection rules
 */
interface LanguagePattern {
  language: string;
  patterns: RegExp[];
  extensions?: string[];
  firstLinePatterns?: RegExp[];
}

// Define patterns for detecting languages
const languagePatterns: LanguagePattern[] = [
  {
    language: 'java',
    patterns: [
      // Java class definitions
      /\b(public|private|protected)\s+(static\s+)?(final\s+)?(class|interface|enum)\s+\w+/,
      // Java imports
      /\bimport\s+[a-z0-9_\.]+(\.[A-Z][a-z0-9_]*)+;/,
      // Java package
      /\bpackage\s+[a-z0-9_\.]+;/,
      // Java annotations
      /@Override|@SuppressWarnings|@Deprecated/
    ],
    extensions: ['.java']
  },
  {
    language: 'javascript',
    patterns: [
      // JS functions and arrow functions
      /\bfunction\s+\w+\s*\(|const\s+\w+\s*=\s*(\(.*\)|[a-zA-Z0-9_]+)\s*=>/,
      // JS import/export
      /\b(import|export)\s+(\{.*\}|\*|default)\s+from/,
      // JS DOM manipulation
      /document\.querySelector|getElementById|addEventListener/,
      // React hooks
      /\buseState\(|useEffect\(|useRef\(|useContext\(/
    ],
    extensions: ['.js', '.jsx', '.mjs']
  },
  {
    language: 'typescript',
    patterns: [
      // TypeScript type definitions
      /\binterface\s+\w+|type\s+\w+\s*=/,
      // TypeScript annotations
      /\w+\s*:\s*(string|number|boolean|any|unknown|object|\{|\[|\w+\[\])/,
      // TypeScript imports with types
      /import\s+type\s+\{.*\}\s+from/,
      // Generic syntax
      /<[A-Z][a-zA-Z0-9]*>/
    ],
    extensions: ['.ts', '.tsx']
  },
  {
    language: 'html',
    patterns: [
      // HTML doctype or tags
      /<!DOCTYPE\s+html>|<html.*>|<body.*>|<head.*>/i,
      // Common HTML elements
      /<(div|span|p|a|img|ul|ol|li|table)[\s>]/i,
      // HTML attributes
      /<[a-z]+\s+[^>]*?(class|style|id|href|src)=/i
    ],
    extensions: ['.html', '.htm'],
    firstLinePatterns: [
      /<!DOCTYPE\s+html>/i
    ]
  },
  {
    language: 'css',
    patterns: [
      // CSS selectors and properties
      /[\.\#]?[a-z0-9_\-]+\s*\{[^}]*:[^}]*\}/i,
      // CSS media queries
      /@media\s+/i,
      // CSS animations
      /@keyframes\s+/i
    ],
    extensions: ['.css']
  },
  {
    language: 'json',
    patterns: [
      // JSON structure starts with { and consists of "key": value pairs
      /^\s*\{\s*"[^"]+"\s*:\s*["{[0-9tfn]/,
      // JSON arrays
      /^\s*\[\s*(\{|"|\d|true|false|null)/
    ],
    extensions: ['.json']
  },
  {
    language: 'yaml',
    patterns: [
      // YAML key-value pairs
      /^\s*[a-zA-Z0-9_-]+\s*:\s*[^\s{[].*$/m,
      // YAML lists
      /^\s*-\s+[a-zA-Z0-9_]/m,
      // YAML document start
      /^---(\s.*)?$/m
    ],
    extensions: ['.yaml', '.yml']
  },
  {
    language: 'python',
    patterns: [
      // Python function definitions
      /\bdef\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(/,
      // Python class definitions
      /\bclass\s+[A-Z][a-zA-Z0-9_]*(\([^)]*\))?:/,
      // Python imports
      /\b(import|from)\s+[a-zA-Z_][a-zA-Z0-9_.]*/,
      // Python indentation
      /^\s{2,}[a-zA-Z0-9_]+/m
    ],
    extensions: ['.py']
  },
  {
    language: 'sql',
    patterns: [
      // SQL queries
      /\b(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE)\s+/i,
      // SQL join operations
      /\bJOIN\s+[a-zA-Z_][a-zA-Z0-9_]*\s+(ON|USING)/i,
      // SQL where clauses
      /\bWHERE\s+[a-zA-Z_][a-zA-Z0-9_]*\s*(=|<|>|<=|>=|<>|!=|LIKE|IN|BETWEEN)/i,
      // SQL functions
      /\b(COUNT|SUM|AVG|MIN|MAX)\s*\(/i
    ],
    extensions: ['.sql']
  },
  {
    language: 'shell',
    patterns: [
      // Shell shebang
      /^#!\/bin\/(bash|sh|zsh)/,
      // Shell variable assignments
      /^\s*[A-Za-z_][A-Za-z0-9_]*=/m,
      // Shell if statements
      /\bif\s+\[\s+.*\s+\];\s+then/,
      // Shell commands with pipes or redirections
      /\|\s*grep|>\s*\/dev\/null/,
      // Common shell commands
      /\b(echo|cd|ls|mkdir|rm|cp|mv)\b/
    ],
    extensions: ['.sh', '.bash']
  },
  {
    language: 'xml',
    patterns: [
      // XML declaration
      /^<\?xml\s+version/i,
      // XML tags with namespace
      /<[a-z0-9_-]+:[a-z0-9_-]+\s+/i,
      // XML elements with attributes
      /<[a-z0-9_-]+\s+[^>]*?(xmlns:|xml:|\w+:)/i
    ],
    extensions: ['.xml'],
    firstLinePatterns: [
      /^<\?xml\s+version/i
    ]
  },
  {
    language: 'c',
    patterns: [
      // C preprocessor directives
      /^\s*#include\s+<[a-zA-Z0-9_\.\/]+\.h>/m,
      // C function definitions
      /^[a-zA-Z0-9_]+\s+[a-zA-Z0-9_]+\s*\([^;{]*\)\s*\{/m,
      // C structs
      /\bstruct\s+[a-zA-Z0-9_]+\s*\{/,
      // C standard libraries
      /#include\s+<(stdio|stdlib|string|math|ctype)\.h>/
    ],
    extensions: ['.c', '.h']
  },
  {
    language: 'cpp',
    patterns: [
      // C++ namespaces
      /\bnamespace\s+[a-zA-Z0-9_]+/,
      // C++ classes with inheritance
      /\bclass\s+[a-zA-Z0-9_]+(\s*:\s*(public|protected|private)\s+[a-zA-Z0-9_]+)?\s*\{/,
      // C++ templates
      /template\s*<(typename|class)\s+[a-zA-Z0-9_]+>/,
      // C++ streams
      /\b(cout|cin|cerr)\s*<<|>>/,
      // C++ std usage
      /\bstd::[a-zA-Z0-9_]+/
    ],
    extensions: ['.cpp', '.cc', '.hpp']
  },
  {
    language: 'ruby',
    patterns: [
      // Ruby class definitions
      /\bclass\s+[A-Z][a-zA-Z0-9_]*(\s*<\s*[A-Z][a-zA-Z0-9_:]*)?/,
      // Ruby method definitions
      /\bdef\s+[a-zA-Z0-9_]+/,
      // Ruby symbols
      /\b:[a-zA-Z0-9_]+/,
      // Ruby blocks
      /\bdo\s*\|[^|]*\|\s*$/m
    ],
    extensions: ['.rb']
  },
  {
    language: 'dockerfile',
    patterns: [
      // Dockerfile instructions
      /^(FROM|RUN|CMD|LABEL|EXPOSE|ENV|ADD|COPY|ENTRYPOINT|VOLUME|USER|WORKDIR|ARG|ONBUILD|STOPSIGNAL|HEALTHCHECK|SHELL)\s+/mi,
      // Multiple RUN commands
      /\bRUN\s+apt-get\s+|RUN\s+npm\s+|RUN\s+pip\s+/i
    ],
    extensions: ['Dockerfile'],
    firstLinePatterns: [
      /^FROM\s+/i
    ]
  },
  {
    language: 'ini',
    patterns: [
      // INI section headers
      /^\s*\[[a-zA-Z0-9_\.-]+\]\s*$/m,
      // INI key-value pairs
      /^\s*[a-zA-Z0-9_\.]+\s*=\s*.*$/m
    ],
    extensions: ['.ini', '.cfg', '.conf']
  },
  {
    language: 'properties',
    patterns: [
      // Java properties format
      /^[a-zA-Z0-9_\.-]+\s*=\s*.*$/m,
      // Properties comments
      /^#.*$/m
    ],
    extensions: ['.properties', '.prop']
  }
];

/**
 * Detects the language of code based on its content
 * @param code The code to analyze
 * @returns The detected language name or 'plaintext' if no match
 */
export function detectLanguage(code: string): string {
  if (!code || code.trim().length === 0) {
    return 'plaintext';
  }

  // Check for specific file extensions in the content (e.g., for config files)
  const configFileMatch = code.match(/filename="([^"]+)"|name="([^"]+)"|^application\.(properties|ya?ml)$/m);
  if (configFileMatch) {
    const filename = configFileMatch[1] || configFileMatch[2] || configFileMatch[0];
    
    if (filename.match(/\.properties$/i)) {
      return 'properties';
    }
    if (filename.match(/\.(yaml|yml)$/i)) {
      return 'yaml';
    }
    if (filename.match(/\.json$/i)) {
      return 'json';
    }
  }
  
  // Special case for application.properties or application.yaml
  if (code.match(/^(server\.port|spring\.datasource|logging\.level)/m)) {
    return 'properties';
  }
  
  // Special case for yaml config files
  if (code.match(/^(apiVersion|kind|metadata|spec):/m)) {
    return 'yaml';
  }

  // Check for specific language patterns
  const scores: Record<string, number> = {};
  
  // Get the first few lines (for more precise detection)
  const firstLines = code.split('\n').slice(0, 10).join('\n');
  
  languagePatterns.forEach(langPattern => {
    // Initialize score
    scores[langPattern.language] = 0;
    
    // Check first line patterns (highest priority)
    if (langPattern.firstLinePatterns) {
      langPattern.firstLinePatterns.forEach(pattern => {
        if (firstLines.match(pattern)) {
          scores[langPattern.language] += 10; // High score for first line matches
        }
      });
    }
    
    // Check regular patterns
    langPattern.patterns.forEach(pattern => {
      const matches = code.match(new RegExp(pattern, 'g'));
      if (matches) {
        scores[langPattern.language] += matches.length;
      }
    });
  });

  // Find the language with the highest score
  let bestMatch = 'plaintext';
  let highestScore = 0;
  
  Object.entries(scores).forEach(([language, score]) => {
    if (score > highestScore) {
      highestScore = score;
      bestMatch = language;
    }
  });
  
  // Only return languages supported by Monaco editor
  return languages.includes(bestMatch as never) ? bestMatch : 'plaintext';
}

// Special handling for common file extensions or names
export function detectLanguageFromFileName(fileName: string): string | null {
  if (!fileName) return null;
  
  const lowerFileName = fileName.toLowerCase();
  
  if (lowerFileName.endsWith('.js')) return 'javascript';
  if (lowerFileName.endsWith('.ts')) return 'typescript';
  if (lowerFileName.endsWith('.jsx')) return 'javascript';
  if (lowerFileName.endsWith('.tsx')) return 'typescript';
  if (lowerFileName.endsWith('.html')) return 'html';
  if (lowerFileName.endsWith('.css')) return 'css';
  if (lowerFileName.endsWith('.json')) return 'json';
  if (lowerFileName.endsWith('.yml') || lowerFileName.endsWith('.yaml')) return 'yaml';
  if (lowerFileName.endsWith('.py')) return 'python';
  if (lowerFileName.endsWith('.java')) return 'java';
  if (lowerFileName.endsWith('.c')) return 'c';
  if (lowerFileName.endsWith('.cpp') || lowerFileName.endsWith('.cc')) return 'cpp';
  if (lowerFileName.endsWith('.rb')) return 'ruby';
  if (lowerFileName.endsWith('.go')) return 'go';
  if (lowerFileName.endsWith('.php')) return 'php';
  if (lowerFileName.endsWith('.sql')) return 'sql';
  if (lowerFileName.endsWith('.sh')) return 'shell';
  if (lowerFileName.endsWith('.md')) return 'markdown';
  if (lowerFileName === 'dockerfile' || lowerFileName.endsWith('.dockerfile')) return 'dockerfile';
  if (lowerFileName.endsWith('.xml')) return 'xml';
  if (lowerFileName.endsWith('.properties') || lowerFileName === 'application.properties') return 'properties';
  
  return null;
} 