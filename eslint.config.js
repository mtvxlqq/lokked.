import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: [
      "dist/**",
      "src-tauri/target/**",
      "src-tauri/gen/**",
      "coverage/**",
      // Референсные макеты из Claude Design: чужой сгенерированный код,
      // хранится как есть.
      "docs/designs/mockups/**",
    ],
  },

  js.configs.recommended,
  tseslint.configs.recommended,

  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2022,
      globals: globals.browser,
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],

      // Дизайн-токены обязательны. Захардкоженный цвет или размер — это способ
      // завести сорок слегка разных чёрных, поэтому в className запрещены
      // произвольные значения с цветом или абсолютной длиной (`text-[#0A0A0B]`,
      // `p-[13px]`), а в коде — голые hex-строки. Всё нужное есть в
      // `src/styles/tokens.css`; чего нет — добавляется туда, а не по месту.
      //
      // Произвольные значения без единиц (`grid-cols-[repeat(...)]`,
      // `w-[calc(...)]`) правило намеренно пропускает: это вёрстка, а не токен.
      "no-restricted-syntax": [
        "error",
        {
          selector:
            "JSXAttribute[name.name='className'] Literal[value=/-\\[(#|rgb|hsl|oklch|[0-9]+(\\.[0-9]+)?(px|rem|em))/]",
          message:
            "Произвольное значение вместо токена. Возьми утилиту из src/styles/tokens.css.",
        },
        {
          selector:
            "JSXAttribute[name.name='className'] TemplateElement[value.raw=/-\\[(#|rgb|hsl|oklch|[0-9]+(\\.[0-9]+)?(px|rem|em))/]",
          message:
            "Произвольное значение вместо токена. Возьми утилиту из src/styles/tokens.css.",
        },
        {
          selector: "Literal[value=/^#[0-9a-fA-F]{3,8}$/]",
          message:
            "Захардкоженный цвет. Возьми токен из src/styles/tokens.css.",
        },
      ],
    },
  },

  {
    files: ["*.{js,ts}", "src/test/**/*.ts"],
    languageOptions: {
      globals: globals.node,
    },
  },

  // Must stay last: turns off every rule Prettier already handles.
  prettier,
);
