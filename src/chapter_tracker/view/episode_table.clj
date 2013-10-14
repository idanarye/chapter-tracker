(ns chapter-tracker.view.episode-table (:gen-class)
  (:use chapter-tracker.view.tools)
  (:use chapter-tracker.model)
)
(import
  '(java.awt Dimension)
  '(javax.swing JTable JButton)
  '(javax.swing.table DefaultTableModel TableCellRenderer)
  '(javax.swing.event ListSelectionListener)
)

(def episode-table-columns [
                            {:caption "Id"             :field :episode-id       :editable false}
                            {:caption "Volume"         :field :volume-number    :editable true}
                            {:caption "Episode"        :field :episode-number   :editable true}
                            {:caption "Name"           :field :episode-name     :editable true}
                            {:caption "DateOfRead"     :field :date-of-read     :editable false}
                           ])

(def episode-table-captions (to-array (map :caption episode-table-columns)))

(defn make-table-updating-lambda [table row]
  (fn [column new-value]
    (let [column-number (condp = column
                          :episode-number 1
                          :episode-name   2
                          :date-of-read   3
                          :date-of-read   3
                          false
                        )]
      (if column-number (.. table getModel (setValueAt new-value row column-number)))
    )
  )
)

(defn create-episodes-table [update-episode-panel-function]
  (let [table (JTable. (proxy [DefaultTableModel] []
                         ;(isCellEditable [row column] (-> episode-table-columns (nth column) :editable true?))
                         (isCellEditable [row column] false)
                       ))]
    (.. table getModel (setColumnIdentifiers episode-table-captions))
    (.. table getSelectionModel (addListSelectionListener (proxy [ListSelectionListener] []
                                                            (valueChanged [e]
                                                              (try
                                                                (update-episode-panel-function (fetch-episode-record
                                                                                                 (.. table getModel (getValueAt (.getSelectedRow table) 0)))
                                                                                               (make-table-updating-lambda table (.getSelectedRow table))
                                                                )
                                                                (catch Exception e)
                                                              )
                                                            )
                                                          )))
    ;(.setPreferredSize table (Dimension. 100 100))
    table
  )
)

(defn update-episodes-table [episodes-table to-series]
  (..
    episodes-table
    getModel
    (setDataVector
      (to-array-2d
        (map (fn [episode]
               (to-array (map #(% episode) (map :field episode-table-columns)))
             )
             ;(sort-by #(vector (or (:volume-number %) 0) (:episode-number %)) #(if (every? number? (list %1 %2))
             (sort-by #(vector (- (or (:volume-number %) 0)) (- (:episode-number %)))
                      ;(fn [[v1 c1] [v2 c2]]
                        ;(cond
                          ;(= (number? v1) (number? v2)) (if (not= v1 v2) (> v1 v2)
                                                          ;(if (every? number? (concat c1 c2))
                                                            ;(> c1 c2)
                                                            ;(number? c2)
                                                          ;))
                          ;(number? v1) false
                          ;:else true
                      ;))
                      (fetch-episode-records-for to-series)
             )))
      episode-table-captions
    )
  )
)
