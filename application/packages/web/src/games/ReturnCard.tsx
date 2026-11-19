import type { ReturnCardModel } from './returnCard';

type Props = {
  card: ReturnCardModel;
  /** Data URL or asset URL for the session frame. */
  frameSrc?: string | null;
};

/**
 * Return card above a game page: last line, what next, days since, frame.
 * Colors follow the Design tab (surface / accent / muted).
 */
export function ReturnCard({ card, frameSrc }: Props) {
  const daysLabel =
    card.daysSince === 0
      ? 'Today'
      : card.daysSince === 1
        ? '1 day ago'
        : `${card.daysSince} days ago`;

  return (
    <section
      className="debrief-return-card"
      data-testid="debrief-return-card"
      aria-label={`Return card for ${card.title}`}
    >
      <div className="debrief-return-card__main">
        <p className="debrief-return-card__line" data-testid="return-card-line">
          {card.line || '—'}
        </p>
        {card.whatNext ? (
          <p className="debrief-return-card__next" data-testid="return-card-next">
            Next: {card.whatNext}
          </p>
        ) : null}
        <p className="debrief-return-card__days" data-testid="return-card-days">
          {daysLabel}
        </p>
      </div>
      {frameSrc ? (
        <img
          className="debrief-return-card__frame"
          data-testid="return-card-frame"
          src={frameSrc}
          alt=""
          width={160}
          height={90}
        />
      ) : null}
    </section>
  );
}
