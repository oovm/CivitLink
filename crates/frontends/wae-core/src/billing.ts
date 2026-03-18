/**
 * 计费维度：四元矩阵 (输入文本, 输出文本, 输入像素, 输出像素)
 */
export interface BillingDimensions {
    /** 输入文本长度 (如字符数或 Token 数) */
    inputText: number;
    /** 输出文本长度 */
    outputText: number;
    /** 输入像素数 (宽 * 高) */
    inputPixels: number;
    /** 输出像素数 */
    outputPixels: number;
}

/**
 * 文本成本定义 (每百万 Tokens/字符)
 */
export interface TextCost {
    /** 每百万单位的价格 */
    perMillion: number;
}

/**
 * 图像成本定义 (每百万像素)
 */
export interface ImageCost {
    /** 每百万像素的价格 */
    perMillion: number;
}

/**
 * 四元矩阵成本配置
 */
export interface BillingCostConfig {
    /** 输入文本成本 (每百万 Tokens) */
    inputText: TextCost;
    /** 输出文本成本 (每百万 Tokens) */
    outputText: TextCost;
    /** 输入图像成本 (每百万像素) */
    inputPixels: ImageCost;
    /** 输出图像成本 (每百万像素) */
    outputPixels: ImageCost;
}

/**
 * 计算总成本
 * @param usage 计费维度
 * @param config 成本配置
 * @returns 总成本
 */
export function calculateTotalCost(usage: BillingDimensions, config: BillingCostConfig): number {
    const million = 1_000_000;
    let total = 0;

    if (usage.inputText > 0) {
        total += (usage.inputText / million) * config.inputText.perMillion;
    }
    if (usage.outputText > 0) {
        total += (usage.outputText / million) * config.outputText.perMillion;
    }
    if (usage.inputPixels > 0) {
        total += (usage.inputPixels / million) * config.inputPixels.perMillion;
    }
    if (usage.outputPixels > 0) {
        total += (usage.outputPixels / million) * config.outputPixels.perMillion;
    }

    return total;
}
