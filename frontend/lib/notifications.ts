import { AlertPriority, AlertType } from './alert-rules';

/** Notification Record Interface */
export interface NotificationAlert {
  id: string;
  type: AlertType;
  priority: AlertPriority;
  message: string;
  timestamp: number;
  data?: Record<string, unknown>;
}

/**
 * Notification Service
 * Dispatches alerts to dashboard, console, and webhooks.
 */
class NotificationService {
  private static instance: NotificationService;
  private subscribers: ((alert: NotificationAlert) => void)[] = [];

  private constructor() {}

  public static getInstance(): NotificationService {
    if (!NotificationService.instance) {
      NotificationService.instance = new NotificationService();
    }
    return NotificationService.instance;
  }

  /** Send an alert to all dispatchers */
  public async send(alert: NotificationAlert): Promise<void> {
    const timestampMs = Date.now();

    // 1. Log to Console (Internal Monitoring)
    this.logToConsole(alert);

    // 2. Dispatch to Subscribed UI Components
    this.subscribers.forEach((sub) => sub(alert));

    // 3. (Mock) Dispatch to Slack/Email if priority is HIGH/CRITICAL
    if (alert.priority === 'HIGH' || alert.priority === 'CRITICAL') {
      await this.dispatchExternal(alert);
    }
  }

  /** Subscribe to new alerts (e.g., from Dashboard) */
  public subscribe(callback: (alert: NotificationAlert) => void): () => void {
    this.subscribers.push(callback);
    return () => {
      this.subscribers = this.subscribers.filter((s) => s !== callback);
    };
  }

  private logToConsole(alert: NotificationAlert): void {
    const icon = alert.priority === 'CRITICAL' ? '🔥' : alert.priority === 'HIGH' ? '🚨' : '⚠️';
    console.log(`[Astera Alert ${icon}] [${alert.priority}] ${alert.type}: ${alert.message}`);
    if (alert.data) {
      console.log('Context Data:', JSON.stringify(alert.data, null, 2));
    }
  }

  /** Mock External Webhook Dispatcher */
  private async dispatchExternal(alert: NotificationAlert): Promise<void> {
    const webhookUrl = process.env.NEXT_PUBLIC_SLACK_WEBHOOK_URL;
    if (!webhookUrl) {
      console.warn(
        '[Astera] Skipping external webhook: No URL provided in env (NEXT_PUBLIC_SLACK_WEBHOOK_URL).',
      );
      return;
    }

    try {
      const payload = {
        text: `*${alert.priority} ALERT: ${alert.type}*`,
        attachments: [
          {
            color: alert.priority === 'CRITICAL' ? '#ff0000' : '#ffa500',
            fields: [
              { title: 'Message', value: alert.message, short: false },
              { title: 'Time', value: new Date(alert.timestamp).toISOString(), short: true },
              { title: 'Context', value: JSON.stringify(alert.data || {}), short: false },
            ],
          },
        ],
      };

      await fetch(webhookUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      console.log(`[Astera] Successfully dispatched ${alert.priority} alert to external webhook.`);
    } catch (error) {
      console.error('[Astera] Failed to dispatch webhook:', error);
    }
  }
}

export const notificationService = NotificationService.getInstance();
