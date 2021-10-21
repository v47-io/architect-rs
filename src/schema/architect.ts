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
}

export type Question = SimpleQuestion | SelectionQuestion;

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

export interface ConditionalFiles {
    /**
     * The condition that decides whether the matched files are created.
     *
     * This is an expression that is handled by handlebars
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
     * Can be multiple names concatenated using `.` to create hierarchial structures in
     * the context.
     *
     * Format: `^[a-zA-Z_$][a-zA-Z0-9_$]*$`
     */
    name: string;
    /**
     * The type of the question which informs the expected values
     */
    type: QuestionType;
    /**
     * A properly spelled out question to ask instead of just presenting the name when
     * processing input
     */
    pretty?: string;
}

export enum QuestionType {
    Identifier = 'Identifier',
    Option = 'Option',
    Selection = 'Selection',
    Text = 'Text'
}
