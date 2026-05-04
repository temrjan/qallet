/**
 * Stub for `.css` imports in Jest.
 *
 * NativeWind v4 expects `import './global.css'` in App.tsx, but Jest's
 * default transform pipeline does not parse CSS. This empty module
 * intercepts the import so test suites can require App.tsx without
 * choking on the @tailwind directives.
 */
module.exports = {};
