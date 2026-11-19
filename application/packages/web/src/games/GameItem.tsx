import { useMemo } from 'react';
import type { LocalNote } from '../db';
import { FreeformItem } from '../freeform/FreeformItem';
import { emptyFreeformBody } from '../freeform/page';
import './returnCard.css';
import { parseGamePageBody } from './page';
import { ReturnCard } from './ReturnCard';
import {
  FIXTURE_AS_OF_DAY,
  FIXTURE_GAME_ID,
  fixtureFrameDataUrl,
  fixtureReturnCard,
} from './fixtureJournal';
import type { ReturnCardModel } from './returnCard';

type Props = {
  note: LocalNote;
  flushRef?: React.MutableRefObject<(() => void) | null>;
  readOnly?: boolean;
  onBodyChange?: (id: string, body: string) => void;
  /** Override the derived card (tests / live journal). */
  card?: ReturnCardModel | null;
  frameSrc?: string | null;
  /** Freeform body for the game note page (journal note later). */
  freeformBody?: string;
  onFreeformChange?: (body: string) => void;
  asOfDay?: string;
};

/**
 * Game page: return card on top, freeform canvas below.
 * Step 6 uses the bundled fixture when the note points at the fixture game id.
 */
export function GameItem(props: Props) {
  const parsed = parseGamePageBody(props.note.body);
  const gameId = parsed?.gameId ?? '';

  const card = useMemo(() => {
    if (props.card) return props.card;
    if (gameId === FIXTURE_GAME_ID) {
      return fixtureReturnCard(props.asOfDay ?? FIXTURE_AS_OF_DAY);
    }
    return {
      gameId,
      title: props.note.title || 'Game',
      line: '',
      whatNext: '',
      daysSince: 0,
      frameId: null,
      framePath: null,
      sessionId: null,
      stoppedAt: null,
    } satisfies ReturnCardModel;
  }, [props.card, props.asOfDay, gameId, props.note.title]);

  const frameSrc =
    props.frameSrc
    ?? (card.frameId && gameId === FIXTURE_GAME_ID ? fixtureFrameDataUrl() : null);

  const freeformNote: LocalNote = {
    ...props.note,
    body: props.freeformBody ?? emptyFreeformBody(),
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col" data-testid="debrief-game-page">
      <ReturnCard card={card} frameSrc={frameSrc} />
      <div className="flex min-h-0 flex-1 flex-col">
        <FreeformItem
          note={freeformNote}
          flushRef={props.flushRef}
          readOnly={props.readOnly ?? false}
          onBodyChange={(_id, body) => {
            props.onFreeformChange?.(body);
          }}
        />
      </div>
    </div>
  );
}
