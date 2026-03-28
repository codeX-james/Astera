import { NextResponse } from 'next/server';
import { monitorService } from '../../../lib/monitoring';

export const dynamic = 'force-dynamic';

export async function GET() {
  try {
    const events = await monitorService.pollEvents();

    return NextResponse.json({
      status: 'success',
      events: events.length,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error('Monitor API Error:', error);
    return NextResponse.json(
      { status: 'error', message: 'Failed to poll events' },
      { status: 500 },
    );
  }
}
