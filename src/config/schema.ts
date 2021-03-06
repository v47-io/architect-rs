/*
 * BSD 3-Clause License
 *
 * Copyright (c) 2021, Alex Katlein
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice, this
 *    list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its
 *    contributors may be used to endorse or promote products derived from
 *    this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

/**
 * The configuration used by Architect when creating an instance of this project template.
 *
 * Everything (including the file itself) is optional, but Architect makes more sense to
 * use when actually configured
 */
export interface Config {
    /**
     * The name of the template.
     *
     * Can be used in handlebars templates using `__template__.name`
     */
    name?: string;
    /**
     * The version of the template.
     *
     * Can be used in handlebars templates using `__template__.version`
     */
    version?: string;
    /**
     * Questions to ask the user to specify dynamic context values.
     *
     * These values are then available in handlebars templates
     */
    questions?: Question[];
    /**
     * Contains multiple filters to control which files are actually considered and rendered
     */
    filters?: Filters;
}

export type Question = SimpleQuestion | SelectionQuestion | CustomQuestion;

export interface SimpleQuestion extends BaseQuestion {
    type: QuestionType.Identifier | QuestionType.Option | QuestionType.Text;
}

export interface SelectionQuestion extends BaseQuestion {
    type: QuestionType.Selection;

    /**
     * The items available for selection.
     *
     * These will be set to `true` in the context if selected.
     *
     * Format: `^[a-zA-Z_$][a-zA-Z0-9_$]*$`
     */
    items: string[];
    /**
     * Specifies whether multiple items can be selected
     */
    multi?: boolean;
}

export interface CustomQuestion extends BaseQuestion {
    type: QuestionType.Custom;

    /**
     * The regular expression that is used to validate the input for this question.
     *
     * When specifying a default value it must match this regular expression
     */
    format: string;
}

/**
 * This interface specifies the configuration properties that decide which files are considered
 * for Handlebars rendering or even included in the target directory
 */
export interface Filters {
    /**
     * Specifies conditions for certain files to be created.
     *
     * These conditions have full access to the context that is created by the questions.
     *
     * Note that conditions specified here don't apply to hidden files that weren't explicitly
     * included using `includeHidden` or files excluded using `exclude`
     */
    conditionalFiles?: ConditionalFiles[];
    /**
     * Specifies Glob expressions to include hidden files in the target.
     *
     * Note that including the `.git` directory here will have no effect
     */
    includeHidden?: string[];
    /**
     * Specifies Glob expressions to exclude files in the target.
     *
     * Note that exclusions have a higher precedence than inclusions and conditional files
     */
    exclude?: string[];
    /**
     * Specifies Glob expressions that indicate the files that should be rendered using Handlebars.
     *
     * This disables Handlebars rendering for all other files. Directory or file names are not affected
     */
    templates?: string[];
    /**
     * Specifies Glob expressions that indicate files that should not be rendered using Handlebars.
     *
     * This property has no effect, if `templates` is also specified
     */
    nonTemplates?: string[];
}

export interface ConditionalFiles {
    /**
     * The condition that decides whether the matched files are created.
     *
     * This is an expression that is handled by handlebars.
     *
     * The expression is automatically wrapped in curly braces (`{{` `}}`) so you
     * only need to specify the actual content of the expression here
     */
    condition: string;
    /**
     * A Glob string specifying the files affected by the condition
     */
    matcher: string;
}

interface BaseQuestion {
    /**
     * The name in the context for the value specified when answering this question.
     *
     * Can be multiple names concatenated using `.` to create hierarchical structures in
     * the context.
     *
     * Format: `^[a-zA-Z_$][a-zA-Z0-9_$]*$`
     */
    name: string;
    /**
     * The type of the question, which indicates the expected values
     */
    type: QuestionType;
    /**
     * A properly spelled out question to ask instead of just presenting the name when
     * processing input
     */
    pretty?: string;
    /**
     * The default answer for this question.
     *
     * If the question is of type `Option`, this should specify a boolean, if it's 'Selection'
     * you can specify either a string or a list of strings, otherwise just a string.
     *
     * Note: Specifying a list of strings will only be accepted if the `Selection` question
     * allows the selection of multiple items
     */
    default?: string | boolean | string[]
}

export enum QuestionType {
    Identifier = 'Identifier',
    Option = 'Option',
    Selection = 'Selection',
    Text = 'Text',
    Custom = 'Custom'
}
